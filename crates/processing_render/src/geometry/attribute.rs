use std::ops::Range;

use bevy::{
    mesh::{Indices, MeshVertexAttribute, VertexAttributeValues},
    prelude::*,
    render::render_resource::VertexFormat,
};

use crate::error::{ProcessingError, Result};

use super::{Geometry, hash_attr_name};

fn clamp_range(range: Range<usize>, len: usize) -> Range<usize> {
    range.start.min(len)..range.end.min(len)
}

fn get_mesh<'a>(
    entity: Entity,
    geometries: &Query<&Geometry>,
    meshes: &'a Assets<Mesh>,
) -> Result<&'a Mesh> {
    let geometry = geometries
        .get(entity)
        .map_err(|_| ProcessingError::GeometryNotFound)?;
    meshes
        .get(&geometry.handle)
        .ok_or(ProcessingError::GeometryNotFound)
}

fn get_mesh_mut<'a>(
    entity: Entity,
    geometries: &Query<&Geometry>,
    meshes: &'a mut Assets<Mesh>,
) -> Result<&'a mut Mesh> {
    let geometry = geometries
        .get(entity)
        .map_err(|_| ProcessingError::GeometryNotFound)?;
    meshes
        .get_mut(&geometry.handle)
        .ok_or(ProcessingError::GeometryNotFound)
}

macro_rules! impl_getter {
    ($name:ident, $attr:expr, $variant:ident, $type:ty) => {
        pub fn $name(
            In((entity, range)): In<(Entity, Range<usize>)>,
            geometries: Query<&Geometry>,
            meshes: Res<Assets<Mesh>>,
        ) -> Result<Vec<$type>> {
            let mesh = get_mesh(entity, &geometries, &meshes)?;
            match mesh.attribute($attr) {
                Some(VertexAttributeValues::$variant(data)) => {
                    Ok(data[clamp_range(range, data.len())].to_vec())
                }
                Some(_) => Err(ProcessingError::InvalidArgument(
                    concat!("Unexpected ", stringify!($name), " format").into(),
                )),
                None => Err(ProcessingError::GeometryNotFound),
            }
        }
    };
}

impl_getter!(get_positions, Mesh::ATTRIBUTE_POSITION, Float32x3, [f32; 3]);
impl_getter!(get_normals, Mesh::ATTRIBUTE_NORMAL, Float32x3, [f32; 3]);
impl_getter!(get_colors, Mesh::ATTRIBUTE_COLOR, Float32x4, [f32; 4]);
impl_getter!(get_uvs, Mesh::ATTRIBUTE_UV_0, Float32x2, [f32; 2]);

pub fn get_indices(
    In((entity, range)): In<(Entity, Range<usize>)>,
    geometries: Query<&Geometry>,
    meshes: Res<Assets<Mesh>>,
) -> Result<Vec<u32>> {
    let mesh = get_mesh(entity, &geometries, &meshes)?;
    match mesh.indices() {
        Some(Indices::U32(data)) => Ok(data[clamp_range(range, data.len())].to_vec()),
        Some(Indices::U16(data)) => {
            let range = clamp_range(range, data.len());
            Ok(data[range].iter().map(|&i| i as u32).collect())
        }
        None => Ok(Vec::new()),
    }
}

macro_rules! impl_setter {
    ($name:ident, $attr:expr, $variant:ident, [$($arg:ident: $arg_ty:ty),+], $arr:expr) => {
        pub fn $name(
            In((entity, index, $($arg),+)): In<(Entity, u32, $($arg_ty),+)>,
            geometries: Query<&Geometry>,
            mut meshes: ResMut<Assets<Mesh>>,
        ) -> Result<()> {
            let mesh = get_mesh_mut(entity, &geometries, &mut meshes)?;
            match mesh.attribute_mut($attr) {
                Some(VertexAttributeValues::$variant(data)) => {
                    let idx = index as usize;
                    if idx < data.len() {
                        data[idx] = $arr;
                        Ok(())
                    } else {
                        Err(ProcessingError::InvalidArgument(format!(
                            "Index {} out of bounds (count: {})", index, data.len()
                        )))
                    }
                }
                Some(_) => Err(ProcessingError::InvalidArgument(
                    concat!("Unexpected ", stringify!($name), " format").into(),
                )),
                None => Err(ProcessingError::InvalidArgument(
                    concat!("Geometry missing ", stringify!($attr)).into(),
                )),
            }
        }
    };
}

impl_setter!(set_vertex, Mesh::ATTRIBUTE_POSITION, Float32x3, [x: f32, y: f32, z: f32], [x, y, z]);
impl_setter!(set_normal, Mesh::ATTRIBUTE_NORMAL, Float32x3, [nx: f32, ny: f32, nz: f32], [nx, ny, nz]);
impl_setter!(set_color, Mesh::ATTRIBUTE_COLOR, Float32x4, [r: f32, g: f32, b: f32, a: f32], [r, g, b, a]);
impl_setter!(set_uv, Mesh::ATTRIBUTE_UV_0, Float32x2, [u: f32, v: f32], [u, v]);

#[derive(Clone, Debug)]
pub enum AttributeValue {
    Float(f32),
    Float2([f32; 2]),
    Float3([f32; 3]),
    Float4([f32; 4]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AttributeFormat {
    Float = 1,
    Float2 = 2,
    Float3 = 3,
    Float4 = 4,
}

impl AttributeFormat {
    pub fn to_vertex_format(self) -> VertexFormat {
        match self {
            Self::Float => VertexFormat::Float32,
            Self::Float2 => VertexFormat::Float32x2,
            Self::Float3 => VertexFormat::Float32x3,
            Self::Float4 => VertexFormat::Float32x4,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Float),
            2 => Some(Self::Float2),
            3 => Some(Self::Float3),
            4 => Some(Self::Float4),
            _ => None,
        }
    }
}

#[derive(Component, Clone)]
pub struct Attribute {
    pub name: &'static str,
    pub format: AttributeFormat,
    pub(crate) inner: MeshVertexAttribute,
}

impl Attribute {
    pub fn new(name: impl Into<String>, format: AttributeFormat) -> Self {
        // we leak here to get a 'static str for the attribute name, but this is okay because
        // we never expect to unload attributes during the lifetime of the application
        // and attribute names are generally small in number
        let name: &'static str = Box::leak(name.into().into_boxed_str());
        let id = hash_attr_name(name);
        let inner = MeshVertexAttribute::new(name, id, format.to_vertex_format());
        Self {
            name,
            format,
            inner,
        }
    }

    pub fn from_builtin(inner: MeshVertexAttribute, format: AttributeFormat) -> Self {
        Self {
            name: inner.name,
            format,
            inner,
        }
    }

    pub fn id(&self) -> u64 {
        hash_attr_name(self.name)
    }
}

#[derive(Resource)]
pub struct BuiltinAttributes {
    pub position: Entity,
    pub normal: Entity,
    pub color: Entity,
    pub uv: Entity,
}

impl FromWorld for BuiltinAttributes {
    fn from_world(world: &mut World) -> Self {
        let position = world
            .spawn(Attribute::from_builtin(
                Mesh::ATTRIBUTE_POSITION,
                AttributeFormat::Float3,
            ))
            .id();
        let normal = world
            .spawn(Attribute::from_builtin(
                Mesh::ATTRIBUTE_NORMAL,
                AttributeFormat::Float3,
            ))
            .id();
        let color = world
            .spawn(Attribute::from_builtin(
                Mesh::ATTRIBUTE_COLOR,
                AttributeFormat::Float4,
            ))
            .id();
        let uv = world
            .spawn(Attribute::from_builtin(
                Mesh::ATTRIBUTE_UV_0,
                AttributeFormat::Float2,
            ))
            .id();

        Self {
            position,
            normal,
            color,
            uv,
        }
    }
}

pub fn create(
    In((name, format)): In<(String, AttributeFormat)>,
    mut commands: Commands,
) -> Result<Entity> {
    // TODO: validation?
    Ok(commands.spawn(Attribute::new(name, format)).id())
}

pub fn destroy(In(entity): In<Entity>, mut commands: Commands) -> Result<()> {
    commands.entity(entity).despawn();
    Ok(())
}

pub fn get_attribute(
    In((entity, attribute_id, index)): In<(Entity, MeshVertexAttribute, u32)>,
    geometries: Query<&Geometry>,
    meshes: Res<Assets<Mesh>>,
) -> Result<AttributeValue> {
    let mesh = get_mesh(entity, &geometries, &meshes)?;
    let idx = index as usize;

    let attr = mesh.attribute(attribute_id).ok_or_else(|| {
        ProcessingError::InvalidArgument(format!(
            "Geometry does not have attribute {}",
            attribute_id.name
        ))
    })?;

    macro_rules! get_idx {
        ($values:expr, $variant:ident) => {
            if idx < $values.len() {
                Ok(AttributeValue::$variant($values[idx]))
            } else {
                Err(ProcessingError::InvalidArgument(format!(
                    "Index {} out of bounds",
                    index,
                )))
            }
        };
    }

    match attr {
        VertexAttributeValues::Float32(v) => get_idx!(v, Float),
        VertexAttributeValues::Float32x2(v) => get_idx!(v, Float2),
        VertexAttributeValues::Float32x3(v) => get_idx!(v, Float3),
        VertexAttributeValues::Float32x4(v) => get_idx!(v, Float4),
        // TODO: handle other formats as needed
        _ => Err(ProcessingError::InvalidArgument(
            "Unsupported attribute format".into(),
        )),
    }
}

pub fn get_attributes(
    In((entity, attribute_id, range)): In<(Entity, MeshVertexAttribute, Range<usize>)>,
    geometries: Query<&Geometry>,
    meshes: Res<Assets<Mesh>>,
) -> Result<Vec<AttributeValue>> {
    let mesh = get_mesh(entity, &geometries, &meshes)?;

    let attr = mesh.attribute(attribute_id).ok_or_else(|| {
        ProcessingError::InvalidArgument(format!(
            "Geometry does not have attribute {}",
            attribute_id.name
        ))
    })?;

    match attr {
        VertexAttributeValues::Float32(v) => Ok(v[clamp_range(range, v.len())]
            .iter()
            .map(|&x| AttributeValue::Float(x))
            .collect()),
        VertexAttributeValues::Float32x2(v) => Ok(v[clamp_range(range, v.len())]
            .iter()
            .map(|&x| AttributeValue::Float2(x))
            .collect()),
        VertexAttributeValues::Float32x3(v) => Ok(v[clamp_range(range, v.len())]
            .iter()
            .map(|&x| AttributeValue::Float3(x))
            .collect()),
        VertexAttributeValues::Float32x4(v) => Ok(v[clamp_range(range, v.len())]
            .iter()
            .map(|&x| AttributeValue::Float4(x))
            .collect()),
        _ => Err(ProcessingError::InvalidArgument(
            "Unsupported attribute format".into(),
        )),
    }
}

pub fn set_attribute(
    In((entity, attribute_id, index, value)): In<(
        Entity,
        MeshVertexAttribute,
        u32,
        AttributeValue,
    )>,
    geometries: Query<&Geometry>,
    mut meshes: ResMut<Assets<Mesh>>,
) -> Result<()> {
    let mesh = get_mesh_mut(entity, &geometries, &mut meshes)?;
    let idx = index as usize;

    let attr = mesh.attribute_mut(attribute_id).ok_or_else(|| {
        ProcessingError::InvalidArgument(format!(
            "Geometry does not have attribute {}",
            attribute_id.name
        ))
    })?;

    macro_rules! set_idx {
        ($values:expr, $v:expr) => {
            if idx < $values.len() {
                $values[idx] = $v;
                Ok(())
            } else {
                Err(ProcessingError::InvalidArgument(format!(
                    "Index {} out of bounds",
                    index,
                )))
            }
        };
    }

    match (attr, value) {
        (VertexAttributeValues::Float32(values), AttributeValue::Float(v)) => set_idx!(values, v),
        (VertexAttributeValues::Float32x2(values), AttributeValue::Float2(v)) => {
            set_idx!(values, v)
        }
        (VertexAttributeValues::Float32x3(values), AttributeValue::Float3(v)) => {
            set_idx!(values, v)
        }
        (VertexAttributeValues::Float32x4(values), AttributeValue::Float4(v)) => {
            set_idx!(values, v)
        }
        _ => Err(ProcessingError::InvalidArgument(
            "Attribute value type does not match attribute format".into(),
        )),
    }
}
