//! See `docs/particles.md`.

pub mod algebra;
pub mod compact;
mod emit;
pub mod grid;
pub mod kernels;
pub mod material;
pub mod pack;
pub mod point_render;
pub mod reduce;
pub mod scan;
mod scatter;
pub mod sort;

pub use algebra::{
    GEN_GAUSSIAN, GEN_SIGNED, GEN_UNIFORM, MAP_ABS, MAP_AFFINE, MAP_CLAMP, MAP_EQ, MAP_FLOOR,
    MAP_GEQ, MAP_GREATER, MAP_LEQ, MAP_LESS, MAP_NEGATE, MAP_NEQ, MAP_SQRT, MAP_SQUARE,
    REDUCE_LENGTH, REDUCE_MAX, REDUCE_MEAN, REDUCE_MIN, REDUCE_SUM, REDUCE_SUMSQ, combine, extract,
    generate, lookup, map, mix, pack, reduce_components,
};
pub use compact::compact;
pub use emit::{
    particles_apply, particles_emit, particles_emit_gpu, particles_flock, particles_gather,
};
pub use grid::{Grid, GridParams, grid_bind, grid_build, grid_create};
pub use kernels::{
    BOUNDS_CLAMP, BOUNDS_REFLECT, BOUNDS_SOFT, BOUNDS_WRAP, COMBINE_ADD, COMBINE_DIV, COMBINE_MAX,
    COMBINE_MIN, COMBINE_MUL, COMBINE_POW, COMBINE_SUB, FALLOFF_CONST, FALLOFF_CUBIC,
    FALLOFF_INVERSE, FALLOFF_LINEAR, FALLOFF_QUADRATIC, FALLOFF_SMOOTHSTEP, particles_kernel_age,
    particles_kernel_attr_combine, particles_kernel_attr_linear, particles_kernel_attr_lookup1d,
    particles_kernel_attr_lookup2d, particles_kernel_attr_mix, particles_kernel_attract,
    particles_kernel_bounds_box, particles_kernel_bounds_geometry, particles_kernel_bounds_sphere,
    particles_kernel_drag, particles_kernel_field, particles_kernel_flock, particles_kernel_force,
    particles_kernel_impulse, particles_kernel_integrate, particles_kernel_noise,
    particles_kernel_orient, particles_kernel_transform, particles_kernel_vortex,
};
pub use reduce::{REDUCE_OP_MAX, REDUCE_OP_MIN, REDUCE_OP_SUM, reduce};
pub use scan::prefix_sum_u32;
pub use scatter::{
    particles_scatter_create, particles_scatter_volume_create, prepare_scatter_source,
    prepare_scatter_volume_source,
};
pub use sort::bitonic_sort_by_key;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::pbr::gpu_instance_batch::GpuInstanceBatchPlugin;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::mesh::allocator::MeshAllocatorSettings;
use bevy::render::render_resource::{BufferDescriptor, BufferUsages};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::ShaderBuffer;

use processing_core::app_mut;
use processing_core::error::{self, ProcessingError, Result};

use crate::compute;
use crate::geometry::{Attribute, AttributeFormat, Geometry, default_attribute_init};

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GpuInstanceBatchPlugin);
        app.add_plugins(pack::ParticlesPackPlugin);
        app.add_plugins(material::ParticlesMaterialPlugin);
        app.add_plugins(kernels::ParticlesKernelsPlugin);
        app.add_plugins(point_render::ParticlesPointRenderPlugin);
    }

    fn finish(&self, app: &mut App) {
        // The mesh allocator emits its GPU buffers before the render device
        // exists, so the STORAGE flag must be set in finish(), not build().
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .world_mut()
            .resource_mut::<MeshAllocatorSettings>()
            .extra_buffer_usages |= BufferUsages::STORAGE;
    }
}

#[derive(Component)]
pub struct Particles {
    pub capacity: u32,
    /// `Attribute` entity to backing `compute::Buffer` entity.
    pub buffers: HashMap<Entity, Entity>,
    /// Must outlive the per-frame draw: `GpuInstanceBatchReservations` queues
    /// mesh batches one frame behind, so respawning per-frame loses the reservation.
    pub draw_entity: Option<Entity>,
    pub raster_draw_entity: Option<Entity>,
    pub connectivity: Option<Connectivity>,
    /// Ring-buffer write cursor; wraps at `capacity`.
    pub emit_head: u32,
}

#[derive(Clone, Copy)]
pub struct Connectivity {
    pub index_buffer: Entity,
    pub indirect_buffer: Entity,
}

impl Particles {
    pub fn buffer(&self, attribute: Entity) -> Option<Entity> {
        self.buffers.get(&attribute).copied()
    }
}

#[derive(Component, Clone, Copy)]
pub struct ParticlesDraw {
    pub particles: Entity,
}

pub fn create(
    In((capacity, attribute_entities)): In<(u32, Vec<Entity>)>,
    mut commands: Commands,
    attributes: Query<&Attribute>,
    mut shader_buffers: ResMut<Assets<ShaderBuffer>>,
    render_device: Res<RenderDevice>,
) -> Result<Entity> {
    let mut buffers = HashMap::with_capacity(attribute_entities.len());
    for attr_entity in attribute_entities {
        let attr = attributes
            .get(attr_entity)
            .map_err(|_| ProcessingError::InvalidEntity)?;
        let per_element = convention_seed_bytes(attr.name, attr.format);
        let elem_size = attr.format.byte_size();
        if per_element.len() != elem_size {
            return Err(ProcessingError::InvalidArgument(format!(
                "attribute '{}' reuses a builtin name with an incompatible format; \
                 declare it with an explicit default",
                attr.name
            )));
        }
        let initial = tile_seed(&per_element, capacity as usize);
        let buffer_entity =
            make_buffer(&mut commands, &mut shader_buffers, &render_device, &initial);
        buffers.insert(attr_entity, buffer_entity);
    }

    let entity = commands
        .spawn(Particles {
            capacity,
            buffers,
            draw_entity: None,
            raster_draw_entity: None,
            connectivity: None,
            emit_head: 0,
        })
        .id();
    Ok(entity)
}

pub fn create_from_geometry(
    In((geom_entity, attribute_entities)): In<(Entity, Vec<Entity>)>,
    mut commands: Commands,
    geometries: Query<&Geometry>,
    attributes: Query<&Attribute>,
    meshes: Res<Assets<Mesh>>,
    mut shader_buffers: ResMut<Assets<ShaderBuffer>>,
    render_device: Res<RenderDevice>,
) -> Result<Entity> {
    let geom = geometries
        .get(geom_entity)
        .map_err(|_| ProcessingError::GeometryNotFound)?;
    let mesh = meshes
        .get(&geom.handle)
        .ok_or(ProcessingError::GeometryNotFound)?;
    let capacity = mesh.count_vertices() as u32;

    let mut buffers = HashMap::with_capacity(attribute_entities.len());
    for attr_entity in attribute_entities {
        let attr = attributes
            .get(attr_entity)
            .map_err(|_| ProcessingError::InvalidEntity)?;
        let byte_size = capacity as u64 * attr.format.byte_size() as u64;

        let initial = mesh
            .attribute(attr.inner)
            .and_then(|values| attribute_values_to_bytes(values, attr.format))
            .filter(|bytes| bytes.len() == byte_size as usize)
            .unwrap_or_else(|| {
                let per_element = convention_seed_bytes(attr.name, attr.format);
                if per_element.len() == attr.format.byte_size() {
                    tile_seed(&per_element, capacity as usize)
                } else {
                    vec![0u8; byte_size as usize]
                }
            });

        let buffer_entity =
            make_buffer(&mut commands, &mut shader_buffers, &render_device, &initial);
        buffers.insert(attr_entity, buffer_entity);
    }

    let connectivity = mesh.indices().map(|indices| {
        let index_data: Vec<u32> = match indices {
            Indices::U16(v) => v.iter().map(|&i| i as u32).collect(),
            Indices::U32(v) => v.clone(),
        };
        let index_buffer = make_buffer_with_usage(
            &mut commands,
            &mut shader_buffers,
            &render_device,
            &u32s_to_bytes(&index_data),
            BufferUsages::INDEX,
        );
        let args = [index_data.len() as u32, 1, 0, 0, 0];
        let indirect_buffer = make_buffer_with_usage(
            &mut commands,
            &mut shader_buffers,
            &render_device,
            &u32s_to_bytes(&args),
            BufferUsages::INDIRECT,
        );
        Connectivity {
            index_buffer,
            indirect_buffer,
        }
    });

    let entity = commands
        .spawn(Particles {
            capacity,
            buffers,
            draw_entity: None,
            raster_draw_entity: None,
            connectivity,
            emit_head: 0,
        })
        .id();
    Ok(entity)
}

pub fn particles_set_connectivity(
    particles_entity: Entity,
    index_count: u32,
) -> error::Result<Entity> {
    use bevy::render::render_resource::BufferUsages;

    let index_buffer =
        crate::buffer_create_with_usage(index_count as u64 * 4, BufferUsages::INDEX)?;
    let indirect_buffer =
        crate::buffer_create_with_usage(20, BufferUsages::INDIRECT | BufferUsages::STORAGE)?;
    crate::buffer_write(indirect_buffer, u32s_to_bytes(&[index_count, 1, 0, 0, 0]))?;

    let previous = app_mut(|app| {
        let mut field = app
            .world_mut()
            .get_mut::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        Ok(field.connectivity.replace(Connectivity {
            index_buffer,
            indirect_buffer,
        }))
    })?;
    if let Some(previous) = previous {
        crate::buffer_destroy(previous.index_buffer)?;
        crate::buffer_destroy(previous.indirect_buffer)?;
    }
    Ok(index_buffer)
}

pub fn particles_connectivity_indirect(particles_entity: Entity) -> error::Result<Entity> {
    app_mut(|app| {
        let field = app
            .world()
            .get::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        field
            .connectivity
            .as_ref()
            .map(|c| c.indirect_buffer)
            .ok_or_else(|| {
                error::ProcessingError::InvalidArgument(
                    "particles have no index buffer; call index_buffer() first".to_string(),
                )
            })
    })
}

pub fn particles_reset_indices(particles_entity: Entity) -> error::Result<()> {
    let indirect = particles_connectivity_indirect(particles_entity)?;
    crate::buffer_write(indirect, u32s_to_bytes(&[0, 1, 0, 0, 0]))
}

fn make_buffer(
    commands: &mut Commands,
    shader_buffers: &mut Assets<ShaderBuffer>,
    render_device: &RenderDevice,
    initial: &[u8],
) -> Entity {
    make_buffer_with_usage(
        commands,
        shader_buffers,
        render_device,
        initial,
        BufferUsages::empty(),
    )
}

fn make_buffer_with_usage(
    commands: &mut Commands,
    shader_buffers: &mut Assets<ShaderBuffer>,
    render_device: &RenderDevice,
    initial: &[u8],
    extra_usage: BufferUsages,
) -> Entity {
    let byte_size = initial.len() as u64;
    let mut shader_buffer = ShaderBuffer::new(initial, RenderAssetUsages::all());
    shader_buffer.buffer_description.usage |= extra_usage;
    let handle = shader_buffers.add(shader_buffer);
    let readback = render_device.create_buffer(&BufferDescriptor {
        label: Some("Particles Buffer Readback"),
        size: byte_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    commands
        .spawn(compute::Buffer {
            handle,
            readback_buffer: readback,
            size: byte_size,
            synced: true,
            bound_rw: false,
        })
        .id()
}

fn u32s_to_bytes(values: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for &value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn attribute_values_to_bytes(
    values: &VertexAttributeValues,
    format: AttributeFormat,
) -> Option<Vec<u8>> {
    match (format, values) {
        (AttributeFormat::Float, VertexAttributeValues::Float32(v)) => {
            Some(v.iter().flat_map(|f| f.to_le_bytes()).collect())
        }
        (AttributeFormat::Float2, VertexAttributeValues::Float32x2(v)) => Some(
            v.iter()
                .flat_map(|p| p.iter().flat_map(|f| f.to_le_bytes()))
                .collect(),
        ),
        (AttributeFormat::Float3, VertexAttributeValues::Float32x3(v)) => Some(
            v.iter()
                .flat_map(|p| p.iter().flat_map(|f| f.to_le_bytes()))
                .collect(),
        ),
        (AttributeFormat::Float4, VertexAttributeValues::Float32x4(v)) => Some(
            v.iter()
                .flat_map(|p| p.iter().flat_map(|f| f.to_le_bytes()))
                .collect(),
        ),
        _ => None,
    }
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    particles: Query<&Particles>,
) -> Result<()> {
    let p = particles
        .get(entity)
        .map_err(|_| ProcessingError::ParticlesNotFound)?;
    for &buffer_entity in p.buffers.values() {
        commands.entity(buffer_entity).despawn();
    }
    if let Some(draw_entity) = p.draw_entity {
        commands.entity(draw_entity).despawn();
    }
    if let Some(raster_draw_entity) = p.raster_draw_entity {
        commands.entity(raster_draw_entity).despawn();
    }
    if let Some(connectivity) = p.connectivity {
        commands.entity(connectivity.index_buffer).despawn();
        commands.entity(connectivity.indirect_buffer).despawn();
    }
    commands.entity(entity).despawn();
    Ok(())
}

pub enum AttributeSeed {
    Ensure,
    Declare(Option<Vec<u8>>),
}

fn tile_seed(per_element: &[u8], capacity: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(capacity * per_element.len());
    for _ in 0..capacity {
        bytes.extend_from_slice(per_element);
    }
    bytes
}

fn convention_seed_bytes(name: &str, format: AttributeFormat) -> Vec<u8> {
    default_attribute_init(name, format)
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

pub fn materialize_attribute(
    In((particles_entity, attribute_entity, seed)): In<(Entity, Entity, AttributeSeed)>,
    mut commands: Commands,
    mut particles_q: Query<&mut Particles>,
    attributes: Query<&Attribute>,
    mut shader_buffers: ResMut<Assets<ShaderBuffer>>,
    render_device: Res<RenderDevice>,
) -> Result<Entity> {
    let attr = attributes
        .get(attribute_entity)
        .map_err(|_| ProcessingError::InvalidEntity)?
        .clone();

    let capacity = {
        let particles = particles_q
            .get(particles_entity)
            .map_err(|_| ProcessingError::ParticlesNotFound)?;
        let existing = match particles.buffers.get(&attribute_entity).copied() {
            Some(buf) => Some(buf),
            None => {
                let mut hit = None;
                for (&e, &buf) in &particles.buffers {
                    let Ok(other) = attributes.get(e) else {
                        continue;
                    };
                    if other.name == attr.name {
                        if other.format != attr.format {
                            return Err(ProcessingError::InvalidArgument(format!(
                                "attribute '{}' already present with a different format",
                                attr.name
                            )));
                        }
                        hit = Some(buf);
                        break;
                    }
                }
                hit
            }
        };
        if let Some(buf) = existing {
            return match seed {
                AttributeSeed::Ensure => Ok(buf),
                AttributeSeed::Declare(_) => Err(ProcessingError::InvalidArgument(format!(
                    "particles already have attribute '{}'",
                    attr.name
                ))),
            };
        }
        particles.capacity as usize
    };

    let elem_size = attr.format.byte_size();
    let per_element = match &seed {
        AttributeSeed::Declare(Some(default_bytes)) => {
            if default_bytes.len() != elem_size {
                return Err(ProcessingError::InvalidArgument(format!(
                    "default value byte size {} does not match attribute '{}' format byte size {}",
                    default_bytes.len(),
                    attr.name,
                    elem_size,
                )));
            }
            default_bytes.clone()
        }
        AttributeSeed::Ensure | AttributeSeed::Declare(None) => {
            let seed = convention_seed_bytes(attr.name, attr.format);
            if seed.len() != elem_size {
                return Err(ProcessingError::InvalidArgument(format!(
                    "attribute '{}' reuses a builtin name with an incompatible format; \
                     declare it with an explicit default",
                    attr.name
                )));
            }
            seed
        }
    };

    let initial = tile_seed(&per_element, capacity);
    let buffer_entity = make_buffer(&mut commands, &mut shader_buffers, &render_device, &initial);
    particles_q
        .get_mut(particles_entity)
        .map_err(|_| ProcessingError::ParticlesNotFound)?
        .buffers
        .insert(attribute_entity, buffer_entity);
    Ok(buffer_entity)
}

pub fn particles_create(capacity: u32, attribute_entities: Vec<Entity>) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(create, (capacity, attribute_entities))
            .unwrap()
    })
}

pub fn particles_create_from_geometry(
    geometry_entity: Entity,
    attribute_entities: Vec<Entity>,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(create_from_geometry, (geometry_entity, attribute_entities))
            .unwrap()
    })
}

pub fn particles_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(destroy, entity)
            .unwrap()
    })
}

pub fn particles_capacity(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world()
            .get::<Particles>(entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?
            .capacity)
    })
}

pub fn particles_buffer(entity: Entity, attribute_entity: Entity) -> error::Result<Option<Entity>> {
    app_mut(|app| {
        Ok(app
            .world()
            .get::<Particles>(entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?
            .buffer(attribute_entity))
    })
}

pub fn particles_ensure_attribute(
    particles_entity: Entity,
    attribute_entity: Entity,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                materialize_attribute,
                (particles_entity, attribute_entity, AttributeSeed::Ensure),
            )
            .unwrap()
    })
}

pub fn particles_attribute_add(
    particles_entity: Entity,
    attribute_entity: Entity,
    default: Option<crate::shader_value::ShaderValue>,
) -> error::Result<()> {
    let default_bytes = match default {
        None => None,
        Some(v) => Some(v.to_bytes().ok_or_else(|| {
            error::ProcessingError::InvalidArgument(
                "default must be a scalar/vector ShaderValue, not a Buffer/Texture/Mesh*"
                    .to_string(),
            )
        })?),
    };
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                materialize_attribute,
                (
                    particles_entity,
                    attribute_entity,
                    AttributeSeed::Declare(default_bytes),
                ),
            )
            .unwrap()
            .map(|_| ())
    })
}
