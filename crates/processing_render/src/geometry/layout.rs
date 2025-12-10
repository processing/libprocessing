use bevy::{mesh::MeshVertexAttribute, prelude::*};

use super::{Attribute, BuiltinAttributes};

// bevy requires an attribute id for each unique vertex attribute. we don't really want to
// expose this to users, so we hash the attribute name to generate a unique id. in theory
// there could be collisions, but in practice this should be fine?
// https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function#FNV-1a_hash
pub const fn hash_attr_name(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut hash = 0xcbf29ce484222325u64;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        i += 1;
    }
    hash
}

/// Describes the layout of vertex attributes within a mesh's vertex buffer.
#[derive(Component, Clone, Debug)]
pub struct VertexLayout {
    attributes: Vec<VertexAttribute>,
}

/// Represents a single vertex attribute within a vertex layout, including its type and offset
/// within the vertex buffer.
#[derive(Clone, Debug)]
pub struct VertexAttribute {
    pub attribute: MeshVertexAttribute,
    pub offset: u64,
}

impl VertexLayout {
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    pub fn default_layout() -> Self {
        let attrs = [
            Mesh::ATTRIBUTE_POSITION,
            Mesh::ATTRIBUTE_NORMAL,
            Mesh::ATTRIBUTE_COLOR,
            Mesh::ATTRIBUTE_UV_0,
        ];

        let mut offset = 0u64;
        let mut layout_attrs = Vec::with_capacity(attrs.len());

        for attr in attrs {
            let size = attr.format.size();
            layout_attrs.push(VertexAttribute { attribute: attr, offset });
            offset += size;
        }

        Self {
            attributes: layout_attrs,
        }
    }

    pub fn attributes(&self) -> &[VertexAttribute] {
        &self.attributes
    }

    pub fn has_attribute(&self, attr: &MeshVertexAttribute) -> bool {
        self.attributes.iter().any(|a| a.attribute.id == attr.id)
    }
}

impl Default for VertexLayout {
    fn default() -> Self {
        Self::default_layout()
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct VertexAttributes: u32 {
        const POSITION = 0x01;
        const NORMAL = 0x02;
        const COLOR = 0x04;
        const UV = 0x08;
    }
}

impl VertexAttributes {
    pub fn to_layout(self) -> VertexLayout {
        let mut attrs = vec![Mesh::ATTRIBUTE_POSITION];

        if self.contains(VertexAttributes::NORMAL) {
            attrs.push(Mesh::ATTRIBUTE_NORMAL);
        }
        if self.contains(VertexAttributes::COLOR) {
            attrs.push(Mesh::ATTRIBUTE_COLOR);
        }
        if self.contains(VertexAttributes::UV) {
            attrs.push(Mesh::ATTRIBUTE_UV_0);
        }

        let mut offset = 0u64;
        let mut layout_attrs = Vec::with_capacity(attrs.len());

        for attr in attrs {
            let size = attr.format.size();
            layout_attrs.push(VertexAttribute { attribute: attr, offset });
            offset += size;
        }

        VertexLayout {
            attributes: layout_attrs,
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct VertexLayoutBuilder {
    attributes: Vec<Entity>,
}

pub fn create(In(()): In<()>, mut commands: Commands) -> Entity {
    commands.spawn(VertexLayoutBuilder::default()).id()
}

pub fn add_position(world: &mut World, entity: Entity) {
    let builtins = world.resource::<BuiltinAttributes>();
    let position = builtins.position;
    if let Some(mut builder) = world.get_mut::<VertexLayoutBuilder>(entity) {
        builder.attributes.push(position);
    }
}

pub fn add_normal(world: &mut World, entity: Entity) {
    let builtins = world.resource::<BuiltinAttributes>();
    let normal = builtins.normal;
    if let Some(mut builder) = world.get_mut::<VertexLayoutBuilder>(entity) {
        builder.attributes.push(normal);
    }
}

pub fn add_color(world: &mut World, entity: Entity) {
    let builtins = world.resource::<BuiltinAttributes>();
    let color = builtins.color;
    if let Some(mut builder) = world.get_mut::<VertexLayoutBuilder>(entity) {
        builder.attributes.push(color);
    }
}

pub fn add_uv(world: &mut World, entity: Entity) {
    let builtins = world.resource::<BuiltinAttributes>();
    let uv = builtins.uv;
    if let Some(mut builder) = world.get_mut::<VertexLayoutBuilder>(entity) {
        builder.attributes.push(uv);
    }
}

pub fn add_attribute(world: &mut World, layout_entity: Entity, attr_entity: Entity) {
    if let Some(mut builder) = world.get_mut::<VertexLayoutBuilder>(layout_entity) {
        builder.attributes.push(attr_entity);
    }
}

pub fn destroy(In(entity): In<Entity>, mut commands: Commands) {
    commands.entity(entity).despawn();
}

pub fn build(
    In(entity): In<Entity>,
    mut commands: Commands,
    builders: Query<&VertexLayoutBuilder>,
    attrs: Query<&Attribute>,
) -> bool {
    let Ok(builder) = builders.get(entity) else {
        return false;
    };

    let mut offset = 0u64;
    let mut layout_attrs = Vec::with_capacity(builder.attributes.len());

    for &attr_entity in &builder.attributes {
        let Ok(attr) = attrs.get(attr_entity) else {
            return false;
        };
        let size = attr.inner.format.size();
        layout_attrs.push(VertexAttribute {
            attribute: attr.inner.clone(),
            offset,
        });
        offset += size;
    }

    let layout = VertexLayout {
        attributes: layout_attrs,
    };

    commands.entity(entity).remove::<VertexLayoutBuilder>().insert(layout);
    true
}
