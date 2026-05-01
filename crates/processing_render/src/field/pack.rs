//! Pack pass — bridges a [`Field`]'s `position` / `rotation` / `scale` PBuffers into the
//! upstream `mesh_input_buffer[base..base+capacity].world_from_local` slots reserved by the
//! entity's [`GpuBatchedMesh3d`].
//!
//! The pack shader is specialized via shader_defs (`HAS_ROTATION`, `HAS_SCALE`) based on
//! which builtin attributes the field carries. Pipelines and bind-group layouts are cached
//! per shader_def combination.

use std::num::NonZeroU64;

use bevy::core_pipeline::Core3d;
use bevy::pbr::{
    MeshCullingDataBuffer, MeshInputUniform, MeshUniform, early_gpu_preprocess,
    gpu_instance_batch::GpuInstanceBatchReservations,
};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::{
    Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
    batching::gpu_preprocessing::BatchedInstanceBuffers,
    render_asset::RenderAssets,
    render_resource::{
        BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
        BufferBindingType, CachedComputePipelineId, CachedPipelineState, ComputePassDescriptor,
        ComputePipelineDescriptor, PipelineCache, ShaderStages, ShaderType, UniformBuffer,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue},
    storage::{GpuShaderBuffer, ShaderBuffer},
    sync_world::{MainEntity, MainEntityHashMap},
};
use bevy::shader::{Shader, ShaderDefVal};

use crate::compute;
use crate::geometry::BuiltinAttributes;

use super::{Field, FieldDraw};

const WORKGROUP_SIZE: u32 = 64;

pub struct FieldPackPlugin;

impl Plugin for FieldPackPlugin {
    fn build(&self, app: &mut App) {
        let shader = {
            let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
            shaders.add(Shader::from_wgsl(
                include_str!("pack.wgsl"),
                "processing_render/field/pack.wgsl",
            ))
        };
        app.insert_resource(FieldPackShader(shader.clone()));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .insert_resource(FieldPackShader(shader))
            .init_resource::<ExtractedFieldDraws>()
            .init_resource::<FieldPackPipelines>()
            .init_resource::<FieldPackBindGroups>()
            .add_systems(ExtractSchedule, extract_field_draws)
            .add_systems(
                Render,
                prepare_pack_bind_groups.in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(Core3d, dispatch_pack.before(early_gpu_preprocess));
    }
}

#[derive(Resource, Clone)]
pub struct FieldPackShader(pub Handle<Shader>);

/// Specialization key — controls which `#ifdef`s are set when compiling the pack shader,
/// and which bindings are present in the bind-group layout.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct PackPipelineKey {
    pub has_rotation: bool,
    pub has_scale: bool,
    pub has_dead: bool,
}

pub struct CachedPackPipeline {
    pub bind_group_layout: BindGroupLayoutDescriptor,
    pub pipeline: CachedComputePipelineId,
}

#[derive(Resource, Default)]
pub struct FieldPackPipelines {
    pub by_key: HashMap<PackPipelineKey, CachedPackPipeline>,
}

#[derive(Copy, Clone, Default, ShaderType)]
struct FieldPackParams {
    base_input_index: u32,
    count: u32,
    _pad0: u32,
    _pad1: u32,
}

pub struct ExtractedFieldData {
    pub key: PackPipelineKey,
    pub position: Handle<ShaderBuffer>,
    pub rotation: Option<Handle<ShaderBuffer>>,
    pub scale: Option<Handle<ShaderBuffer>>,
    pub dead: Option<Handle<ShaderBuffer>>,
}

#[derive(Resource, Default)]
pub struct ExtractedFieldDraws {
    pub by_main: MainEntityHashMap<ExtractedFieldData>,
}

#[derive(Resource, Default)]
pub struct FieldPackBindGroups {
    per_batch: MainEntityHashMap<PerBatchBindGroup>,
}

struct PerBatchBindGroup {
    bind_group: BindGroup,
    pipeline: CachedComputePipelineId,
    dispatch_count: u32,
}

fn pack_layout_entries(key: PackPipelineKey) -> Vec<BindGroupLayoutEntry> {
    let storage_rw = BindingType::Buffer {
        ty: BufferBindingType::Storage { read_only: false },
        has_dynamic_offset: false,
        min_binding_size: None,
    };
    let storage_r = BindingType::Buffer {
        ty: BufferBindingType::Storage { read_only: true },
        has_dynamic_offset: false,
        min_binding_size: None,
    };
    let uniform = BindingType::Buffer {
        ty: BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: NonZeroU64::new(16),
    };

    let mut entries = vec![
        layout_entry(0, storage_rw),
        layout_entry(1, storage_rw),
        layout_entry(2, storage_r),
    ];
    if key.has_rotation {
        entries.push(layout_entry(3, storage_r));
    }
    if key.has_scale {
        entries.push(layout_entry(4, storage_r));
    }
    if key.has_dead {
        entries.push(layout_entry(5, storage_r));
    }
    entries.push(layout_entry(6, uniform));
    entries
}

fn layout_entry(binding: u32, ty: BindingType) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty,
        count: None,
    }
}

fn shader_defs_for(key: PackPipelineKey) -> Vec<ShaderDefVal> {
    let mut defs = Vec::new();
    if key.has_rotation {
        defs.push("HAS_ROTATION".into());
    }
    if key.has_scale {
        defs.push("HAS_SCALE".into());
    }
    if key.has_dead {
        defs.push("HAS_DEAD".into());
    }
    defs
}

fn get_or_create_pipeline(
    pipelines: &mut FieldPackPipelines,
    pipeline_cache: &PipelineCache,
    shader: &Handle<Shader>,
    key: PackPipelineKey,
) -> CachedComputePipelineId {
    if let Some(cached) = pipelines.by_key.get(&key) {
        return cached.pipeline;
    }
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        format!(
            "FieldPackBindGroupLayout(rot={},scale={},dead={})",
            key.has_rotation, key.has_scale, key.has_dead
        ),
        &pack_layout_entries(key),
    );
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(
            format!(
                "field_pack_pipeline(rot={},scale={},dead={})",
                key.has_rotation, key.has_scale, key.has_dead
            )
            .into(),
        ),
        layout: vec![bind_group_layout.clone()],
        shader: shader.clone(),
        shader_defs: shader_defs_for(key),
        entry_point: Some("pack".into()),
        ..default()
    });
    pipelines.by_key.insert(
        key,
        CachedPackPipeline {
            bind_group_layout,
            pipeline,
        },
    );
    pipelines.by_key.get(&key).unwrap().pipeline
}

fn extract_field_draws(
    field_draws: Extract<Query<(Entity, &FieldDraw)>>,
    fields: Extract<Query<&Field>>,
    buffers: Extract<Query<&compute::Buffer>>,
    builtins: Extract<Res<BuiltinAttributes>>,
    mut extracted: ResMut<ExtractedFieldDraws>,
) {
    extracted.by_main.clear();
    for (entity, field_draw) in field_draws.iter() {
        let Ok(field) = fields.get(field_draw.field) else {
            continue;
        };
        let Some(pos_pbuf) = field.pbuffer(builtins.position) else {
            continue;
        };
        let Ok(pos_buf) = buffers.get(pos_pbuf) else {
            continue;
        };
        let rotation = field
            .pbuffer(builtins.rotation)
            .and_then(|e| buffers.get(e).ok())
            .map(|b| b.handle.clone());
        let scale = field
            .pbuffer(builtins.scale)
            .and_then(|e| buffers.get(e).ok())
            .map(|b| b.handle.clone());
        let dead = field
            .pbuffer(builtins.dead)
            .and_then(|e| buffers.get(e).ok())
            .map(|b| b.handle.clone());

        let key = PackPipelineKey {
            has_rotation: rotation.is_some(),
            has_scale: scale.is_some(),
            has_dead: dead.is_some(),
        };
        extracted.by_main.insert(
            MainEntity::from(entity),
            ExtractedFieldData {
                key,
                position: pos_buf.handle.clone(),
                rotation,
                scale,
                dead,
            },
        );
    }
}

fn prepare_pack_bind_groups(
    shader: Res<FieldPackShader>,
    mut pipelines: ResMut<FieldPackPipelines>,
    pipeline_cache: Res<PipelineCache>,
    extracted: Res<ExtractedFieldDraws>,
    reservations: Res<GpuInstanceBatchReservations>,
    batched_instance_buffers: Res<BatchedInstanceBuffers<MeshUniform, MeshInputUniform>>,
    culling_data_buffer: Res<MeshCullingDataBuffer>,
    gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut bind_groups: ResMut<FieldPackBindGroups>,
) {
    bind_groups.per_batch.clear();

    let Some(input_buffer) = batched_instance_buffers
        .current_input_buffer
        .buffer()
        .buffer()
    else {
        return;
    };
    let Some(culling_buffer) = culling_data_buffer.buffer() else {
        return;
    };

    for (main_entity, data) in extracted.by_main.iter() {
        let Some(reservation) = reservations.by_entity.get(main_entity) else {
            continue;
        };
        let Some(gpu_position) = gpu_buffers.get(&data.position) else {
            continue;
        };
        let gpu_rotation = data
            .rotation
            .as_ref()
            .and_then(|h| gpu_buffers.get(h));
        if data.key.has_rotation && gpu_rotation.is_none() {
            continue;
        }
        let gpu_scale = data.scale.as_ref().and_then(|h| gpu_buffers.get(h));
        if data.key.has_scale && gpu_scale.is_none() {
            continue;
        }
        let gpu_dead = data.dead.as_ref().and_then(|h| gpu_buffers.get(h));
        if data.key.has_dead && gpu_dead.is_none() {
            continue;
        }

        let pipeline_id =
            get_or_create_pipeline(&mut pipelines, &pipeline_cache, &shader.0, data.key);
        if !matches!(
            pipeline_cache.get_compute_pipeline_state(pipeline_id),
            CachedPipelineState::Ok(_)
        ) {
            continue;
        }
        let cached = pipelines.by_key.get(&data.key).unwrap();

        let params = FieldPackParams {
            base_input_index: reservation.input_buffer_base,
            count: reservation.max_capacity,
            ..default()
        };
        let mut uniform = UniformBuffer::from(params);
        uniform.write_buffer(&render_device, &render_queue);

        let mut entries: Vec<BindGroupEntry> = vec![
            BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: culling_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: gpu_position.buffer.as_entire_binding(),
            },
        ];
        if let Some(gpu_rotation) = gpu_rotation {
            entries.push(BindGroupEntry {
                binding: 3,
                resource: gpu_rotation.buffer.as_entire_binding(),
            });
        }
        if let Some(gpu_scale) = gpu_scale {
            entries.push(BindGroupEntry {
                binding: 4,
                resource: gpu_scale.buffer.as_entire_binding(),
            });
        }
        if let Some(gpu_dead) = gpu_dead {
            entries.push(BindGroupEntry {
                binding: 5,
                resource: gpu_dead.buffer.as_entire_binding(),
            });
        }
        entries.push(BindGroupEntry {
            binding: 6,
            resource: uniform.binding().unwrap(),
        });

        let bind_group = render_device.create_bind_group(
            Some("field_pack_bind_group"),
            &pipeline_cache.get_bind_group_layout(&cached.bind_group_layout),
            &entries,
        );

        let dispatch_count = reservation.max_capacity.div_ceil(WORKGROUP_SIZE);
        bind_groups.per_batch.insert(
            *main_entity,
            PerBatchBindGroup {
                bind_group,
                pipeline: pipeline_id,
                dispatch_count,
            },
        );
    }
}

fn dispatch_pack(
    mut render_context: RenderContext,
    bind_groups: Res<FieldPackBindGroups>,
    pipeline_cache: Res<PipelineCache>,
) {
    if bind_groups.per_batch.is_empty() {
        return;
    }

    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("field_pack"),
            timestamp_writes: None,
        });

    for per_batch in bind_groups.per_batch.values() {
        let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(per_batch.pipeline) else {
            continue;
        };
        pass.set_pipeline(compute_pipeline);
        pass.set_bind_group(0, &per_batch.bind_group, &[]);
        pass.dispatch_workgroups(per_batch.dispatch_count, 1, 1);
    }
}
