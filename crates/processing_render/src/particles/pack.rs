use std::num::NonZeroU64;

use bevy::core_pipeline::Core3d;
use bevy::pbr::{
    MeshCullingDataBuffer, MeshInputUniform, MeshUniform, early_gpu_preprocess,
    gpu_instance_batch::GpuInstanceBatchReservations,
};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::{
    Extract, ExtractSchedule, Render, RenderApp, RenderStartup, RenderSystems,
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

use bevy::pbr::SkinUniforms;
use bevy::render::render_resource::{BufferInitDescriptor, BufferUsages};

use crate::animation::SkinPosePool;
use crate::compute;
use crate::geometry::BuiltinAttributes;

use super::{Particles, ParticlesDraw};

const WORKGROUP_SIZE: u32 = 64;

pub struct ParticlesPackPlugin;

impl Plugin for ParticlesPackPlugin {
    fn build(&self, app: &mut App) {
        let shader = {
            let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
            shaders.add(Shader::from_wgsl(
                include_str!("pack.wgsl"),
                "processing_render/particles/pack.wgsl",
            ))
        };
        app.insert_resource(ParticlesPackShader(shader.clone()));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .insert_resource(ParticlesPackShader(shader))
            .init_resource::<ExtractedParticlesDraws>()
            .init_resource::<ParticlesPackPipelines>()
            .init_resource::<ParticlesPackBindGroups>()
            .add_systems(RenderStartup, prewarm_pack_pipelines)
            .add_systems(ExtractSchedule, extract_particles_draws)
            .add_systems(
                Render,
                prepare_pack_bind_groups.in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(Core3d, dispatch_pack.before(early_gpu_preprocess));
    }
}

#[derive(Resource, Clone)]
pub struct ParticlesPackShader(pub Handle<Shader>);

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct PackPipelineKey {
    pub has_rotation: bool,
    pub has_scale: bool,
    pub has_life: bool,
    pub has_skin: bool,
}

pub struct CachedPackPipeline {
    pub bind_group_layout: BindGroupLayoutDescriptor,
    pub pipeline: CachedComputePipelineId,
}

#[derive(Resource, Default)]
pub struct ParticlesPackPipelines {
    pub by_key: HashMap<PackPipelineKey, CachedPackPipeline>,
}

#[derive(Copy, Clone, Default, ShaderType)]
struct ParticlesPackParams {
    base_input_index: u32,
    count: u32,
    pool_size: u32,
    _pad1: u32,
}

pub struct ExtractedParticlesData {
    pub key: PackPipelineKey,
    pub position: Handle<ShaderBuffer>,
    pub rotation: Option<Handle<ShaderBuffer>>,
    pub scale: Option<Handle<ShaderBuffer>>,
    pub life: Option<Handle<ShaderBuffer>>,
    /// Phase attribute buffer + the pool's slot entities (main world), whose
    /// live skin indices prepare resolves each frame.
    pub skin: Option<(Handle<ShaderBuffer>, Vec<MainEntity>)>,
}

#[derive(Resource, Default)]
pub struct ExtractedParticlesDraws {
    pub by_main: MainEntityHashMap<ExtractedParticlesData>,
}

#[derive(Resource, Default)]
pub struct ParticlesPackBindGroups {
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
    if key.has_life {
        entries.push(layout_entry(5, storage_r));
    }
    entries.push(layout_entry(6, uniform));
    if key.has_skin {
        entries.push(layout_entry(7, storage_r));
        entries.push(layout_entry(8, storage_r));
    }
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
    if key.has_life {
        defs.push("HAS_LIFE".into());
    }
    if key.has_skin {
        defs.push("HAS_SKIN".into());
    }
    defs
}

/// Queue every pack-kernel variant up front.
///
/// The pack kernel is what writes per-instance transforms into the batch slab,
/// so until its pipeline exists an instanced draw renders NOTHING — measured:
/// a particle system drawn from frame 1 is blank on frame 1 and correct on
/// frame 2, while a second system whose first draw happens later appears
/// immediately (the pipeline is hot by then). That one-time cost also lands
/// MID-SHOW the first time a new variant appears — the skinned (`has_skin`)
/// pack for the crow cut is a different pipeline from the unskinned one, so it
/// would otherwise blank its own first frame at the cut.
///
/// There are only 2^4 = 16 combinations and they are small compute pipelines,
/// so compiling them all at startup is cheap and makes instanced draws behave
/// like Processing expects: draw it, and it draws THIS frame.
fn prewarm_pack_pipelines(
    shader: Res<ParticlesPackShader>,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<ParticlesPackPipelines>,
) {
    for bits in 0..16u8 {
        let key = PackPipelineKey {
            has_rotation: bits & 0b0001 != 0,
            has_scale: bits & 0b0010 != 0,
            has_life: bits & 0b0100 != 0,
            has_skin: bits & 0b1000 != 0,
        };
        get_or_create_pipeline(&mut pipelines, &pipeline_cache, &shader.0, key);
    }
}

fn get_or_create_pipeline(
    pipelines: &mut ParticlesPackPipelines,
    pipeline_cache: &PipelineCache,
    shader: &Handle<Shader>,
    key: PackPipelineKey,
) -> CachedComputePipelineId {
    if let Some(cached) = pipelines.by_key.get(&key) {
        return cached.pipeline;
    }
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        format!(
            "ParticlesPackBindGroupLayout(rot={},scale={},life={},skin={})",
            key.has_rotation, key.has_scale, key.has_life, key.has_skin
        ),
        &pack_layout_entries(key),
    );
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(
            format!(
                "particles_pack_pipeline(rot={},scale={},life={},skin={})",
                key.has_rotation, key.has_scale, key.has_life, key.has_skin
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

fn extract_particles_draws(
    particles_draws: Extract<Query<(Entity, &ParticlesDraw)>>,
    particles_q: Extract<Query<&Particles>>,
    buffers: Extract<Query<&compute::Buffer>>,
    pools: Extract<Query<&SkinPosePool>>,
    builtins: Extract<Res<BuiltinAttributes>>,
    mut extracted: ResMut<ExtractedParticlesDraws>,
) {
    extracted.by_main.clear();
    for (entity, particles_draw) in particles_draws.iter() {
        let Ok(p) = particles_q.get(particles_draw.particles) else {
            continue;
        };
        let Some(pos_entity) = p.buffer(builtins.position) else {
            continue;
        };
        let Ok(pos_buf) = buffers.get(pos_entity) else {
            continue;
        };
        let rotation = p
            .buffer(builtins.rotation)
            .and_then(|e| buffers.get(e).ok())
            .map(|b| b.handle.clone());
        let scale = p
            .buffer(builtins.scale)
            .and_then(|e| buffers.get(e).ok())
            .map(|b| b.handle.clone());
        let life = p
            .buffer(builtins.life)
            .and_then(|e| buffers.get(e).ok())
            .map(|b| b.handle.clone());

        let skin = p.skin.and_then(|ps| {
            let phase_entity = p.buffer(ps.phase_attr)?;
            let phase_buf = buffers.get(phase_entity).ok()?;
            let pool = pools.get(ps.pool).ok()?;
            Some((
                phase_buf.handle.clone(),
                pool.slots
                    .iter()
                    .map(|&e| MainEntity::from(e))
                    .collect::<Vec<_>>(),
            ))
        });

        let key = PackPipelineKey {
            has_rotation: rotation.is_some(),
            has_scale: scale.is_some(),
            has_life: life.is_some(),
            has_skin: skin.is_some(),
        };
        extracted.by_main.insert(
            MainEntity::from(entity),
            ExtractedParticlesData {
                key,
                position: pos_buf.handle.clone(),
                rotation,
                scale,
                life,
                skin,
            },
        );
    }
}

fn prepare_pack_bind_groups(
    shader: Res<ParticlesPackShader>,
    mut pipelines: ResMut<ParticlesPackPipelines>,
    pipeline_cache: Res<PipelineCache>,
    extracted: Res<ExtractedParticlesDraws>,
    reservations: Res<GpuInstanceBatchReservations>,
    batched_instance_buffers: Res<BatchedInstanceBuffers<MeshUniform, MeshInputUniform>>,
    culling_data_buffer: Res<MeshCullingDataBuffer>,
    gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>,
    skin_uniforms: Res<SkinUniforms>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut bind_groups: ResMut<ParticlesPackBindGroups>,
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
        let gpu_rotation = data.rotation.as_ref().and_then(|h| gpu_buffers.get(h));
        if data.key.has_rotation && gpu_rotation.is_none() {
            continue;
        }
        let gpu_scale = data.scale.as_ref().and_then(|h| gpu_buffers.get(h));
        if data.key.has_scale && gpu_scale.is_none() {
            continue;
        }
        let gpu_life = data.life.as_ref().and_then(|h| gpu_buffers.get(h));
        if data.key.has_life && gpu_life.is_none() {
            continue;
        }

        // Resolve the pose pool's live skin indices. Offsets can change any
        // frame as skins come and go, so the table is rebuilt per frame; if
        // the pool isn't fully extracted yet (first frame or two), skip the
        // batch rather than draw with garbage joints.
        let mut skin_bindings = None;
        if let Some((phase_handle, slots)) = &data.skin {
            let Some(gpu_phase) = gpu_buffers.get(phase_handle) else {
                continue;
            };
            let mut table: Vec<u32> = Vec::with_capacity(slots.len());
            for slot in slots {
                match skin_uniforms.skin_byte_offset(*slot) {
                    Some(offset) => table.push(offset.index()),
                    None => break,
                }
            }
            if table.len() != slots.len() || table.is_empty() {
                continue;
            }
            let bytes: Vec<u8> = table.iter().flat_map(|v| v.to_le_bytes()).collect();
            let table_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("particles_skin_index_table"),
                contents: &bytes,
                usage: BufferUsages::STORAGE,
            });
            skin_bindings = Some((gpu_phase, table_buffer, table.len() as u32));
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

        let params = ParticlesPackParams {
            base_input_index: reservation.input_buffer_base,
            count: reservation.max_capacity,
            pool_size: skin_bindings.as_ref().map(|(_, _, k)| *k).unwrap_or(0),
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
        if let Some(gpu_life) = gpu_life {
            entries.push(BindGroupEntry {
                binding: 5,
                resource: gpu_life.buffer.as_entire_binding(),
            });
        }
        entries.push(BindGroupEntry {
            binding: 6,
            resource: uniform.binding().unwrap(),
        });
        if let Some((gpu_phase, table_buffer, _)) = &skin_bindings {
            entries.push(BindGroupEntry {
                binding: 7,
                resource: gpu_phase.buffer.as_entire_binding(),
            });
            entries.push(BindGroupEntry {
                binding: 8,
                resource: table_buffer.as_entire_binding(),
            });
        }

        let bind_group = render_device.create_bind_group(
            Some("particles_pack_bind_group"),
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
    bind_groups: Res<ParticlesPackBindGroups>,
    pipeline_cache: Res<PipelineCache>,
) {
    if bind_groups.per_batch.is_empty() {
        return;
    }

    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("particles_pack"),
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
