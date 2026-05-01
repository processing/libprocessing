//! GPU-resident particle / instancing container.
//!
//! A [`Field`] holds a set of named [`PBuffer`](crate::compute::Buffer)s — one per registered
//! attribute. It is pure storage: it carries no instance shape and no material. The shape is
//! supplied at draw time via the `field` verb, and the material is read from ambient state at
//! that point. Rasterization is layered on later by spawning a transient
//! `bevy::pbr::gpu_instance_batch::GpuBatchedMesh3d` entity that consumes the Field's PBuffers
//! through the pack pass.
//!
//! See `docs/field.md` for the full design.

pub mod material;
pub mod pack;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::VertexAttributeValues;
use bevy::pbr::gpu_instance_batch::GpuInstanceBatchPlugin;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::{BufferDescriptor, BufferUsages};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::ShaderBuffer;

use processing_core::error::{ProcessingError, Result};

use crate::compute;
use crate::geometry::{Attribute, AttributeFormat, Geometry};

pub struct FieldPlugin;

impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GpuInstanceBatchPlugin);
        app.add_plugins(pack::FieldPackPlugin);
        app.add_plugins(material::FieldColorMaterialPlugin);
    }
}

/// A GPU-resident container of named per-instance attribute buffers.
///
/// `pbuffers` maps an [`Attribute`](crate::geometry::Attribute) entity to its backing
/// [`compute::Buffer`] entity. The set of registered attributes is fixed at creation.
///
/// `draw_entity` is the persistent rasterization entity carrying `GpuBatchedMesh3d` and
/// the active material — created lazily on the first `field` draw call and reused on
/// subsequent ones. It must persist across frames because the upstream batching queue
/// processes mesh instance batches one frame after the reservation is created; despawning
/// per-frame would lose the entity before it ever gets queued.
///
/// `emit_head` is the ring-buffer write cursor used by `field_emit`. New particles are
/// written to slots `[emit_head, emit_head + n) mod capacity` and the head advances by `n`.
/// When the ring wraps, oldest particles are overwritten — capacity is a visible contract.
#[derive(Component)]
pub struct Field {
    pub capacity: u32,
    pub pbuffers: HashMap<Entity, Entity>,
    pub draw_entity: Option<Entity>,
    pub emit_head: u32,
}

impl Field {
    pub fn pbuffer(&self, attribute: Entity) -> Option<Entity> {
        self.pbuffers.get(&attribute).copied()
    }
}

/// Marker on a transient render entity indicating it rasterizes a [`Field`].
///
/// The pack pass uses this to look up which Field's PBuffers to read when writing
/// per-instance transforms into the upstream `mesh_input_buffer`.
#[derive(Component, Clone, Copy)]
pub struct FieldDraw {
    pub field: Entity,
}

pub fn create(
    In((capacity, attribute_entities)): In<(u32, Vec<Entity>)>,
    mut commands: Commands,
    attributes: Query<&Attribute>,
    mut shader_buffers: ResMut<Assets<ShaderBuffer>>,
    render_device: Res<RenderDevice>,
) -> Result<Entity> {
    let mut pbuffers = HashMap::with_capacity(attribute_entities.len());
    for attr_entity in attribute_entities {
        let attr = attributes
            .get(attr_entity)
            .map_err(|_| ProcessingError::InvalidEntity)?;
        let byte_size = capacity as u64 * attr.format.byte_size() as u64;
        let buffer_entity = make_pbuffer(
            &mut commands,
            &mut shader_buffers,
            &render_device,
            &vec![0u8; byte_size as usize],
        );
        pbuffers.insert(attr_entity, buffer_entity);
    }

    let field_entity = commands
        .spawn(Field {
            capacity,
            pbuffers,
            draw_entity: None,
            emit_head: 0,
        })
        .id();
    Ok(field_entity)
}

/// Create a Field whose capacity matches the source [`Geometry`]'s vertex count
/// and whose PBuffers are pre-seeded from the geometry's mesh attributes where
/// names line up. Any registered attribute the mesh doesn't supply (or whose
/// format doesn't match) gets zero-initialized — the user fills it in via
/// `buffer_write` or `field_emit`.
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

    let mut pbuffers = HashMap::with_capacity(attribute_entities.len());
    for attr_entity in attribute_entities {
        let attr = attributes
            .get(attr_entity)
            .map_err(|_| ProcessingError::InvalidEntity)?;
        let byte_size = capacity as u64 * attr.format.byte_size() as u64;

        let initial = mesh
            .attribute(attr.inner)
            .and_then(|values| attribute_values_to_bytes(values, attr.format))
            .filter(|bytes| bytes.len() == byte_size as usize)
            .unwrap_or_else(|| vec![0u8; byte_size as usize]);

        let buffer_entity =
            make_pbuffer(&mut commands, &mut shader_buffers, &render_device, &initial);
        pbuffers.insert(attr_entity, buffer_entity);
    }

    let field_entity = commands
        .spawn(Field {
            capacity,
            pbuffers,
            draw_entity: None,
            emit_head: 0,
        })
        .id();
    Ok(field_entity)
}

fn make_pbuffer(
    commands: &mut Commands,
    shader_buffers: &mut Assets<ShaderBuffer>,
    render_device: &RenderDevice,
    initial: &[u8],
) -> Entity {
    let byte_size = initial.len() as u64;
    let handle = shader_buffers.add(ShaderBuffer::new(initial, RenderAssetUsages::all()));
    let readback = render_device.create_buffer(&BufferDescriptor {
        label: Some("Field PBuffer Readback"),
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
    fields: Query<&Field>,
) -> Result<()> {
    let field = fields
        .get(entity)
        .map_err(|_| ProcessingError::FieldNotFound)?;
    for &buffer_entity in field.pbuffers.values() {
        commands.entity(buffer_entity).despawn();
    }
    if let Some(draw_entity) = field.draw_entity {
        commands.entity(draw_entity).despawn();
    }
    commands.entity(entity).despawn();
    Ok(())
}
