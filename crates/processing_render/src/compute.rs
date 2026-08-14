use std::collections::{BTreeSet, HashMap};

use bevy::asset::{AssetId, RenderAssetUsages};
use bevy::mesh::MeshVertexAttribute;
use bevy::{
    prelude::*,
    render::{
        RenderApp,
        mesh::{RenderMesh, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_resource::{
            BindGroupLayoutDescriptor, Buffer as WgpuBuffer, BufferDescriptor, BufferUsages,
            CachedComputePipelineId, CachedPipelineState, CommandEncoderDescriptor,
            ComputePassDescriptor, ComputePipelineDescriptor, MapMode, OwnedBindingResource,
            PipelineCache, PollType,
        },
        renderer::{RenderDevice, RenderQueue},
        storage::{GpuShaderBuffer, ShaderBuffer},
        texture::GpuImage,
    },
};

use bevy_naga_reflect::dynamic_shader::DynamicShader;

use crate::geometry::{Attribute, Geometry};
use crate::material::custom::Shader;
use processing_core::error::{ProcessingError, Result};

pub struct ComputePlugin;

impl Plugin for ComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, invalidate_rw_buffers);
    }
}

#[derive(Component)]
pub struct Buffer {
    pub handle: Handle<ShaderBuffer>,
    pub readback_buffer: WgpuBuffer,
    pub size: u64,
    pub synced: bool,
    pub bound_rw: bool,
}

fn readback_buffer(device: &RenderDevice, size: u64) -> WgpuBuffer {
    device.create_buffer(&BufferDescriptor {
        label: Some("Buffer Readback"),
        size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

pub fn create_buffer(
    In(size): In<u64>,
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    render_device: Res<RenderDevice>,
) -> Entity {
    let handle = buffers.add(ShaderBuffer::new(
        &vec![0u8; size as usize],
        RenderAssetUsages::all(),
    ));
    commands
        .spawn(Buffer {
            handle,
            readback_buffer: readback_buffer(&render_device, size),
            size,
            synced: true,
            bound_rw: false,
        })
        .id()
}

pub fn create_buffer_with_usage(
    In((size, extra_usage)): In<(u64, BufferUsages)>,
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    render_device: Res<RenderDevice>,
) -> Entity {
    let mut shader_buffer = ShaderBuffer::new(&vec![0u8; size as usize], RenderAssetUsages::all());
    shader_buffer.buffer_description.usage |= extra_usage;
    let handle = buffers.add(shader_buffer);
    commands
        .spawn(Buffer {
            handle,
            readback_buffer: readback_buffer(&render_device, size),
            size,
            synced: true,
            bound_rw: false,
        })
        .id()
}

pub fn create_buffer_with_data(
    In(data): In<Vec<u8>>,
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    render_device: Res<RenderDevice>,
) -> Entity {
    let size = data.len() as u64;
    let handle = buffers.add(ShaderBuffer::new(&data, RenderAssetUsages::all()));
    commands
        .spawn(Buffer {
            handle,
            readback_buffer: readback_buffer(&render_device, size),
            size,
            synced: true,
            bound_rw: false,
        })
        .id()
}

pub fn write_buffer_cpu(
    In((handle, offset, data)): In<(Handle<ShaderBuffer>, u64, Vec<u8>)>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) -> Result<()> {
    let mut asset = buffers
        .get_mut(&handle)
        .ok_or(ProcessingError::BufferNotFound)?;
    let dst = asset.data.as_mut().ok_or(ProcessingError::BufferNotFound)?;
    let start = offset as usize;
    let end = start + data.len();
    dst[start..end].copy_from_slice(&data);
    Ok(())
}

/// caller must write bytes back via `get_mut_untracked` to avoid triggering
/// a re-upload.
pub fn read_buffer_gpu(
    In((handle, readback_buffer, size)): In<(Handle<ShaderBuffer>, WgpuBuffer, u64)>,
    gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) -> Result<Vec<u8>> {
    let gpu_buffer = &gpu_buffers
        .get(&handle)
        .ok_or(ProcessingError::BufferNotFound)?
        .buffer;

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(gpu_buffer, 0, &readback_buffer, 0, size);
    render_queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = readback_buffer.slice(0..size);
    let (s, r) = crossbeam_channel::bounded(1);
    buffer_slice.map_async(MapMode::Read, move |result| {
        let _ = s.send(result);
    });
    render_device
        .poll(PollType::wait_indefinitely())
        .map_err(|e| ProcessingError::BufferMapError(format!("poll failed: {e}")))?;
    r.recv()
        .map_err(|e| ProcessingError::BufferMapError(format!("map channel closed: {e}")))?
        .map_err(|e| ProcessingError::BufferMapError(format!("map failed: {e}")))?;

    let bytes = buffer_slice
        .get_mapped_range()
        .map_err(|e| ProcessingError::BufferMapError(format!("get_mapped_range failed: {e}")))?
        .to_vec();
    readback_buffer.unmap();
    Ok(bytes)
}

pub fn invalidate_rw_buffers(mut buffers: Query<&mut Buffer>) {
    for mut buf in &mut buffers {
        if buf.bound_rw && buf.synced {
            buf.synced = false;
        }
    }
}

pub fn destroy_buffer(In(entity): In<Entity>, mut commands: Commands) -> Result<()> {
    commands.entity(entity).despawn();
    Ok(())
}

#[derive(Clone, Copy)]
pub enum MeshBindingRef {
    Attribute { geom: Entity, attribute: Entity },
    Index { geom: Entity },
}

#[derive(Clone)]
pub enum ResolvedMeshBinding {
    Attribute {
        mesh_id: AssetId<Mesh>,
        attribute: MeshVertexAttribute,
    },
    Index {
        mesh_id: AssetId<Mesh>,
    },
}

#[derive(Component)]
pub struct Compute {
    pub shader: DynamicShader,
    pub entry_point: String,
    pub pipeline_id: CachedComputePipelineId,
    pub bind_group_layout_descriptors: Vec<(u32, BindGroupLayoutDescriptor)>,
    pub mesh_bindings: HashMap<String, MeshBindingRef>,
}

fn queue_pipeline(
    In(descriptor): In<ComputePipelineDescriptor>,
    pipeline_cache: Res<PipelineCache>,
) -> CachedComputePipelineId {
    pipeline_cache.queue_compute_pipeline(descriptor)
}

fn pump_pipeline(
    In(id): In<CachedComputePipelineId>,
    mut pipeline_cache: ResMut<PipelineCache>,
) -> Result<bool> {
    pipeline_cache.process_queue();
    match pipeline_cache.get_compute_pipeline_state(id) {
        CachedPipelineState::Ok(_) => Ok(true),
        CachedPipelineState::Err(e) => Err(ProcessingError::PipelineCompileError(format!("{e}"))),
        _ => Ok(false),
    }
}

pub fn create_compute(app: &mut App, shader_entity: Entity) -> Result<Entity> {
    let (module, shader_handle) = {
        let program = app
            .world()
            .get::<Shader>(shader_entity)
            .ok_or(ProcessingError::ShaderNotFound)?;
        (program.module.clone(), program.shader_handle.clone())
    };

    let compute_ep = module
        .entry_points
        .iter()
        .find(|ep| ep.stage == naga::ShaderStage::Compute)
        .ok_or_else(|| {
            ProcessingError::ShaderCompilationError(
                "Shader has no @compute entry point".to_string(),
            )
        })?;
    let entry_point = compute_ep.name.clone();

    let mut shader = DynamicShader::new(module).map_err(|e| {
        let mut msg = e.to_string();
        let mut src: &dyn std::error::Error = &e;
        while let Some(s) = src.source() {
            msg.push_str(&format!("\n  caused by: {s}"));
            src = s;
        }
        ProcessingError::ShaderCompilationError(msg)
    })?;
    shader.init();

    let reflection = shader.reflection();
    let groups: BTreeSet<u32> = reflection.parameters().map(|p| p.group()).collect();

    let bind_group_layout_descriptors: Vec<(u32, BindGroupLayoutDescriptor)> = groups
        .iter()
        .map(|&group| {
            let entries = reflection.bind_group_layout(group);
            (
                group,
                BindGroupLayoutDescriptor {
                    label: "compute_bind_group_layout".into(),
                    entries,
                },
            )
        })
        .collect();

    let max_group = groups.iter().last().copied().map_or(0, |g| g + 1);
    let mut layout_descriptors = vec![BindGroupLayoutDescriptor::default(); max_group as usize];
    for (group, desc) in &bind_group_layout_descriptors {
        layout_descriptors[*group as usize] = desc.clone();
    }

    let descriptor = ComputePipelineDescriptor {
        label: Some("processing_compute".into()),
        layout: layout_descriptors,
        immediate_size: 0,
        shader: shader_handle.clone(),
        shader_defs: Vec::new(),
        entry_point: Some(entry_point.clone().into()),
        zero_initialize_workgroup_memory: true,
        constants: default(),
    };

    let pipeline_id = app
        .sub_app_mut(RenderApp)
        .world_mut()
        .run_system_cached_with(queue_pipeline, descriptor)
        .unwrap();

    const MAX_WAIT: u32 = 64;
    for _ in 0..MAX_WAIT {
        app.update();
        let done = app
            .sub_app_mut(RenderApp)
            .world_mut()
            .run_system_cached_with(pump_pipeline, pipeline_id)
            .unwrap()?;
        if done {
            return Ok(app
                .world_mut()
                .spawn(Compute {
                    shader,
                    entry_point,
                    pipeline_id,
                    bind_group_layout_descriptors,
                    mesh_bindings: HashMap::new(),
                })
                .id());
        }
    }
    Err(ProcessingError::PipelineNotReady(MAX_WAIT))
}

pub fn dispatch(
    In((pipeline_id, layout_descriptors, shader, mesh_bindings, x, y, z)): In<(
        CachedComputePipelineId,
        Vec<(u32, BindGroupLayoutDescriptor)>,
        DynamicShader,
        Vec<(String, ResolvedMeshBinding)>,
        u32,
        u32,
        u32,
    )>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    mesh_allocator: Res<MeshAllocator>,
) -> Result<()> {
    let pipeline = pipeline_cache
        .get_compute_pipeline(pipeline_id)
        .ok_or(ProcessingError::PipelineNotReady(0))?
        .clone();

    let reflection = shader.reflection();

    let mut bind_groups = Vec::new();
    for (group, desc) in &layout_descriptors {
        let layout = pipeline_cache.get_bind_group_layout(desc);
        let mut bindings =
            reflection.create_bindings(*group, &shader, &render_device, &gpu_images, &gpu_buffers);

        for (name, resolved) in &mesh_bindings {
            let Some(param) = reflection.parameter(name) else {
                return Err(ProcessingError::UnknownShaderProperty(name.clone()));
            };
            if param.group() != *group {
                continue;
            }
            let buffer = resolve_mesh_binding(resolved, &render_meshes, &mesh_allocator)?;
            let binding_idx = param.binding();
            if let Some(slot) = bindings.iter_mut().find(|(b, _)| *b == binding_idx) {
                slot.1 = OwnedBindingResource::Buffer(buffer);
            } else {
                bindings.push((binding_idx, OwnedBindingResource::Buffer(buffer)));
            }
        }

        let bind_group_entries: Vec<_> = bindings
            .iter()
            .map(
                |(binding, resource)| bevy::render::render_resource::BindGroupEntry {
                    binding: *binding,
                    resource: resource.get_binding(),
                },
            )
            .collect();

        let bind_group = render_device.create_bind_group(
            Some("compute_bind_group"),
            &layout,
            &bind_group_entries,
        );
        bind_groups.push(bind_group);
    }

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("compute_pass"),
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        for ((group, _), bg) in layout_descriptors.iter().zip(bind_groups.iter()) {
            pass.set_bind_group(*group, bg, &[]);
        }
        pass.dispatch_workgroups(x, y, z);
    }
    render_queue.submit(std::iter::once(encoder.finish()));

    Ok(())
}

pub fn destroy_compute(In(entity): In<Entity>, mut commands: Commands) -> Result<()> {
    commands.entity(entity).despawn();
    Ok(())
}

fn resolve_mesh_binding(
    resolved: &ResolvedMeshBinding,
    render_meshes: &RenderAssets<RenderMesh>,
    mesh_allocator: &MeshAllocator,
) -> Result<bevy::render::render_resource::Buffer> {
    match resolved {
        ResolvedMeshBinding::Attribute { mesh_id, attribute } => {
            let render_mesh = render_meshes
                .get(*mesh_id)
                .ok_or(ProcessingError::GeometryNotFound)?;
            let binding_idx = render_mesh
                .layout
                .0
                .binding_index_for_attribute(attribute.id)
                .ok_or_else(|| {
                    ProcessingError::InvalidArgument(format!(
                        "mesh has no `{}` attribute (deinterleave required?)",
                        attribute.name
                    ))
                })?;
            let slice = mesh_allocator
                .mesh_vertex_slice(mesh_id, binding_idx as u8)
                .ok_or_else(|| {
                    ProcessingError::InvalidArgument(format!(
                        "mesh attribute `{}` not yet allocated",
                        attribute.name
                    ))
                })?;
            Ok(slice.buffer.clone())
        }
        ResolvedMeshBinding::Index { mesh_id } => {
            let slice = mesh_allocator.mesh_index_slice(mesh_id).ok_or_else(|| {
                ProcessingError::InvalidArgument(
                    "mesh has no index buffer or it is not yet allocated".to_string(),
                )
            })?;
            Ok(slice.buffer.clone())
        }
    }
}

pub fn resolve_mesh_bindings(
    world: &World,
    compute: &Compute,
) -> Result<Vec<(String, ResolvedMeshBinding)>> {
    let mut out = Vec::with_capacity(compute.mesh_bindings.len());
    for (name, mesh_ref) in &compute.mesh_bindings {
        let resolved = match mesh_ref {
            MeshBindingRef::Attribute { geom, attribute } => {
                let g = world
                    .get::<Geometry>(*geom)
                    .ok_or(ProcessingError::GeometryNotFound)?;
                let a = world
                    .get::<Attribute>(*attribute)
                    .ok_or(ProcessingError::InvalidEntity)?;
                ResolvedMeshBinding::Attribute {
                    mesh_id: g.handle.id(),
                    attribute: a.inner,
                }
            }
            MeshBindingRef::Index { geom } => {
                let g = world
                    .get::<Geometry>(*geom)
                    .ok_or(ProcessingError::GeometryNotFound)?;
                ResolvedMeshBinding::Index {
                    mesh_id: g.handle.id(),
                }
            }
        };
        out.push((name.clone(), resolved));
    }
    Ok(out)
}
