use std::ops::Range;

use bevy::{
    asset::AssetMut,
    mesh::{Indices, MeshVertexAttribute, VertexAttributeValues},
    prelude::*,
    render::render_resource::VertexFormat,
};

use processing_core::error::{ProcessingError, Result};

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
) -> Result<AssetMut<'a, Mesh>> {
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
    ($name:ident, $attr:expr, $variant:ident, $vec_ty:ty) => {
        pub fn $name(
            In((entity, index, value)): In<(Entity, u32, $vec_ty)>,
            geometries: Query<&Geometry>,
            mut meshes: ResMut<Assets<Mesh>>,
        ) -> Result<()> {
            let mut mesh = get_mesh_mut(entity, &geometries, &mut meshes)?;
            match mesh.attribute_mut($attr) {
                Some(VertexAttributeValues::$variant(data)) => {
                    let idx = index as usize;
                    if idx < data.len() {
                        data[idx] = value.to_array();
                        Ok(())
                    } else {
                        Err(ProcessingError::InvalidArgument(format!(
                            "Index {} out of bounds (count: {})",
                            index,
                            data.len()
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

impl_setter!(set_vertex, Mesh::ATTRIBUTE_POSITION, Float32x3, Vec3);
impl_setter!(set_normal, Mesh::ATTRIBUTE_NORMAL, Float32x3, Vec3);
impl_setter!(set_color, Mesh::ATTRIBUTE_COLOR, Float32x4, Vec4);
impl_setter!(set_uv, Mesh::ATTRIBUTE_UV_0, Float32x2, Vec2);

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

    pub fn byte_size(self) -> usize {
        match self {
            Self::Float => 4,
            Self::Float2 => 8,
            Self::Float3 => 12,
            Self::Float4 => 16,
        }
    }

    pub fn components(self) -> usize {
        self.byte_size() / 4
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
        // leaked for a 'static name; attributes are never unloaded and are few.
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

    pub fn from_builtin_with_name(
        name: &'static str,
        inner: MeshVertexAttribute,
        format: AttributeFormat,
    ) -> Self {
        Self {
            name,
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
    pub rotation: Entity,
    pub scale: Entity,
    pub life: Entity,
    pub velocity: Entity,
    pub age: Entity,
}

impl FromWorld for BuiltinAttributes {
    fn from_world(world: &mut World) -> Self {
        let position = world
            .spawn(Attribute::from_builtin_with_name(
                "position",
                Mesh::ATTRIBUTE_POSITION,
                AttributeFormat::Float3,
            ))
            .id();
        let normal = world
            .spawn(Attribute::from_builtin_with_name(
                "normal",
                Mesh::ATTRIBUTE_NORMAL,
                AttributeFormat::Float3,
            ))
            .id();
        let color = world
            .spawn(Attribute::from_builtin_with_name(
                "color",
                Mesh::ATTRIBUTE_COLOR,
                AttributeFormat::Float4,
            ))
            .id();
        let uv = world
            .spawn(Attribute::from_builtin_with_name(
                "uv",
                Mesh::ATTRIBUTE_UV_0,
                AttributeFormat::Float2,
            ))
            .id();
        let rotation = world
            .spawn(Attribute::new("rotation", AttributeFormat::Float4))
            .id();
        let scale = world
            .spawn(Attribute::new("scale", AttributeFormat::Float3))
            .id();
        let life = world
            .spawn(Attribute::new("life", AttributeFormat::Float))
            .id();
        let velocity = world
            .spawn(Attribute::new("velocity", AttributeFormat::Float3))
            .id();
        let age = world
            .spawn(Attribute::new("age", AttributeFormat::Float))
            .id();

        Self {
            position,
            normal,
            color,
            uv,
            rotation,
            scale,
            life,
            velocity,
            age,
        }
    }
}

impl BuiltinAttributes {
    pub fn by_name(&self, name: &str) -> Option<Entity> {
        Some(match name {
            "position" => self.position,
            "normal" => self.normal,
            "color" => self.color,
            "uv" => self.uv,
            "rotation" => self.rotation,
            "scale" => self.scale,
            "life" => self.life,
            "velocity" => self.velocity,
            "age" => self.age,
            _ => return None,
        })
    }
}

pub fn default_attribute_init(name: &str, format: AttributeFormat) -> Vec<f32> {
    match name {
        "life" => vec![1.0],
        "scale" => vec![1.0, 1.0, 1.0],
        "color" => vec![1.0, 1.0, 1.0, 1.0],
        "rotation" => vec![0.0, 0.0, 0.0, 1.0],
        _ => vec![0.0; format.components()],
    }
}

#[derive(Resource, Default)]
pub struct AttributeRegistry {
    by_name: std::collections::HashMap<u64, (Entity, AttributeFormat)>,
}

pub fn create(
    In((name, format)): In<(String, AttributeFormat)>,
    mut commands: Commands,
    builtins: Res<BuiltinAttributes>,
    mut registry: ResMut<AttributeRegistry>,
) -> Result<Entity> {
    if let Some(entity) = builtins.by_name(&name) {
        return Ok(entity);
    }
    let id = hash_attr_name(&name);
    if let Some(&(entity, existing)) = registry.by_name.get(&id) {
        if existing != format {
            return Err(ProcessingError::InvalidArgument(format!(
                "attribute '{name}' was already declared as {existing:?}, cannot \
                 redeclare it as {format:?}"
            )));
        }
        return Ok(entity);
    }
    let entity = commands.spawn(Attribute::new(name, format)).id();
    registry.by_name.insert(id, (entity, format));
    Ok(entity)
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
    let mut mesh = get_mesh_mut(entity, &geometries, &mut meshes)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_defaults_match_format() {
        use AttributeFormat::*;
        let builtins = [
            ("position", Float3),
            ("normal", Float3),
            ("color", Float4),
            ("uv", Float2),
            ("rotation", Float4),
            ("scale", Float3),
            ("life", Float),
            ("velocity", Float3),
            ("age", Float),
        ];
        for (name, format) in builtins {
            assert_eq!(
                default_attribute_init(name, format).len(),
                format.components(),
                "default_attribute_init({name:?}) does not match {format:?} component count",
            );
        }
    }

    #[test]
    fn unknown_name_seeds_zero_for_its_format() {
        for format in [
            AttributeFormat::Float,
            AttributeFormat::Float2,
            AttributeFormat::Float3,
            AttributeFormat::Float4,
        ] {
            let seed = default_attribute_init("custom_thing", format);
            assert_eq!(seed.len(), format.components());
            assert!(seed.iter().all(|&f| f == 0.0));
        }
    }
}
