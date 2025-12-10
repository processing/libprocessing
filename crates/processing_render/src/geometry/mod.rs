//! Geometry is a retained-mode representation of 3D mesh data that can be used for efficient
//! rendering. Typically, Processing's "sketch" API creates new mesh data every frame, which can be
//! inefficient for complex geometries. Geometry is backed by a Bevy [`Mesh`](Mesh) asset.
mod attributes;
pub mod layout;

pub use attributes::*;
pub use layout::{hash_attr_name, VertexAttributes, VertexLayout, VertexAttribute, VertexLayoutBuilder};

use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, Meshable, MeshVertexAttributeId, VertexAttributeValues},
    prelude::*,
    render::render_resource::PrimitiveTopology,
};

use crate::error::{ProcessingError, Result};

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuiltinAttributes>();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Topology {
    PointList = 0,
    LineList = 1,
    LineStrip = 2,
    #[default]
    TriangleList = 3,
    TriangleStrip = 4,
}

impl Topology {
    pub fn to_primitive_topology(self) -> PrimitiveTopology {
        match self {
            Self::PointList => PrimitiveTopology::PointList,
            Self::LineList => PrimitiveTopology::LineList,
            Self::LineStrip => PrimitiveTopology::LineStrip,
            Self::TriangleList => PrimitiveTopology::TriangleList,
            Self::TriangleStrip => PrimitiveTopology::TriangleStrip,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::PointList),
            1 => Some(Self::LineList),
            2 => Some(Self::LineStrip),
            3 => Some(Self::TriangleList),
            4 => Some(Self::TriangleStrip),
            _ => None,
        }
    }
}

#[derive(Component)]
pub struct Geometry {
    pub handle: Handle<Mesh>,
    pub layout: VertexLayout,
    pub current_normal: [f32; 3],
    pub current_color: [f32; 4],
    pub current_uv: [f32; 2],
    pub custom_current: HashMap<MeshVertexAttributeId, AttributeValue>,
}

impl Geometry {
    pub fn new(handle: Handle<Mesh>, layout: VertexLayout) -> Self {
        Self {
            handle,
            layout,
            current_normal: [0.0, 0.0, 1.0],
            current_color: [1.0, 1.0, 1.0, 1.0],
            current_uv: [0.0, 0.0],
            custom_current: HashMap::new(),
        }
    }

    pub fn layout(&self) -> &VertexLayout {
        &self.layout
    }
}

fn create_empty_mesh(layout: &VertexLayout, topology: Topology) -> Mesh {
    let mut mesh = Mesh::new(topology.to_primitive_topology(), RenderAssetUsages::default());

    for attr in layout.attributes() {
        let empty_values = match attr.attribute.format {
            bevy::render::render_resource::VertexFormat::Float32 => {
                VertexAttributeValues::Float32(Vec::new())
            }
            bevy::render::render_resource::VertexFormat::Float32x2 => {
                VertexAttributeValues::Float32x2(Vec::new())
            }
            bevy::render::render_resource::VertexFormat::Float32x3 => {
                VertexAttributeValues::Float32x3(Vec::new())
            }
            bevy::render::render_resource::VertexFormat::Float32x4 => {
                VertexAttributeValues::Float32x4(Vec::new())
            }
            _ => continue,
        };
        mesh.insert_attribute(attr.attribute.clone(), empty_values);
    }

    mesh
}

pub fn create(
    In(topology): In<Topology>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
) -> Entity {
    create_with_layout(In((VertexLayout::default(), topology)), commands, meshes)
}

pub fn create_with_layout(
    In((layout, topology)): In<(VertexLayout, Topology)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) -> Entity {
    let mesh = create_empty_mesh(&layout, topology);
    let handle = meshes.add(mesh);

    commands.spawn(Geometry::new(handle, layout)).id()
}

pub fn create_with_attributes(
    In((attrs, topology)): In<(VertexAttributes, Topology)>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
) -> Entity {
    create_with_layout(In((attrs.to_layout(), topology)), commands, meshes)
}

pub fn create_box(
    In((width, height, depth)): In<(f32, f32, f32)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) -> Entity {
    let cuboid = Cuboid::new(width, height, depth);
    let mesh = cuboid.mesh().build();
    let handle = meshes.add(mesh);

    commands.spawn(Geometry::new(handle, VertexLayout::default())).id()
}

pub fn normal(world: &mut World, entity: Entity, nx: f32, ny: f32, nz: f32) -> Result<()> {
    let mut geometry = world
        .get_mut::<Geometry>(entity)
        .ok_or(ProcessingError::GeometryNotFound)?;
    geometry.current_normal = [nx, ny, nz];
    Ok(())
}

pub fn color(world: &mut World, entity: Entity, r: f32, g: f32, b: f32, a: f32) -> Result<()> {
    let mut geometry = world
        .get_mut::<Geometry>(entity)
        .ok_or(ProcessingError::GeometryNotFound)?;
    geometry.current_color = [r, g, b, a];
    Ok(())
}

pub fn uv(world: &mut World, entity: Entity, u: f32, v: f32) -> Result<()> {
    let mut geometry = world
        .get_mut::<Geometry>(entity)
        .ok_or(ProcessingError::GeometryNotFound)?;
    geometry.current_uv = [u, v];
    Ok(())
}

pub fn attribute(
    world: &mut World,
    geo_entity: Entity,
    attr_entity: Entity,
    value: AttributeValue,
) -> Result<()> {
    let attr = world
        .get::<Attribute>(attr_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let attr_id = attr.inner.id;
    let mut geometry = world
        .get_mut::<Geometry>(geo_entity)
        .ok_or(ProcessingError::GeometryNotFound)?;
    geometry.custom_current.insert(attr_id, value);
    Ok(())
}

pub fn vertex(
    In((entity, x, y, z)): In<(Entity, f32, f32, f32)>,
    mut geometries: Query<&mut Geometry>,
    mut meshes: ResMut<Assets<Mesh>>,
) -> Result<()> {
    let geometry = geometries
        .get_mut(entity)
        .map_err(|_| ProcessingError::GeometryNotFound)?;

    let mesh = meshes
        .get_mut(&geometry.handle)
        .ok_or(ProcessingError::GeometryNotFound)?;

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        positions.push([x, y, z]);
    }

    if geometry.layout.has_attribute(&Mesh::ATTRIBUTE_NORMAL) {
        if let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
        {
            normals.push(geometry.current_normal);
        }
    }

    if geometry.layout.has_attribute(&Mesh::ATTRIBUTE_COLOR) {
        if let Some(VertexAttributeValues::Float32x4(colors)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        {
            colors.push(geometry.current_color);
        }
    }

    if geometry.layout.has_attribute(&Mesh::ATTRIBUTE_UV_0) {
        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            uvs.push(geometry.current_uv);
        }
    }

    for attr in geometry.layout.attributes() {
        if let Some(current) = geometry.custom_current.get(&attr.attribute.id) {
            match (mesh.attribute_mut(attr.attribute.clone()), current) {
                (Some(VertexAttributeValues::Float32(values)), AttributeValue::Float(v)) => {
                    values.push(*v);
                }
                (Some(VertexAttributeValues::Float32x2(values)), AttributeValue::Float2(v)) => {
                    values.push(*v);
                }
                (Some(VertexAttributeValues::Float32x3(values)), AttributeValue::Float3(v)) => {
                    values.push(*v);
                }
                (Some(VertexAttributeValues::Float32x4(values)), AttributeValue::Float4(v)) => {
                    values.push(*v);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

pub fn index(
    In((entity, i)): In<(Entity, u32)>,
    geometries: Query<&Geometry>,
    mut meshes: ResMut<Assets<Mesh>>,
) -> Result<()> {
    let geometry = geometries
        .get(entity)
        .map_err(|_| ProcessingError::GeometryNotFound)?;

    let mesh = meshes
        .get_mut(&geometry.handle)
        .ok_or(ProcessingError::GeometryNotFound)?;

    match mesh.indices_mut() {
        Some(Indices::U32(indices)) => {
            indices.push(i);
        }
        Some(Indices::U16(indices)) => {
            indices.push(i as u16);
        }
        None => {
            mesh.insert_indices(Indices::U32(vec![i]));
        }
    }

    Ok(())
}

pub fn vertex_count(
    In(entity): In<Entity>,
    geometries: Query<&Geometry>,
    meshes: Res<Assets<Mesh>>,
) -> Result<u32> {
    let geometry = geometries
        .get(entity)
        .map_err(|_| ProcessingError::GeometryNotFound)?;
    let mesh = meshes
        .get(&geometry.handle)
        .ok_or(ProcessingError::GeometryNotFound)?;
    Ok(mesh.count_vertices() as u32)
}

pub fn index_count(
    In(entity): In<Entity>,
    geometries: Query<&Geometry>,
    meshes: Res<Assets<Mesh>>,
) -> Result<u32> {
    let geometry = geometries
        .get(entity)
        .map_err(|_| ProcessingError::GeometryNotFound)?;
    let mesh = meshes
        .get(&geometry.handle)
        .ok_or(ProcessingError::GeometryNotFound)?;
    Ok(mesh.indices().map(|i| i.len() as u32).unwrap_or(0))
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    geometries: Query<&Geometry>,
    mut meshes: ResMut<Assets<Mesh>>,
) -> Result<()> {
    let geometry = geometries
        .get(entity)
        .map_err(|_| ProcessingError::GeometryNotFound)?;

    meshes.remove(&geometry.handle);
    commands.entity(entity).despawn();
    Ok(())
}
