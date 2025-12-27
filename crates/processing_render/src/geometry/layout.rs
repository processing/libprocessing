use bevy::prelude::*;

use super::BuiltinAttributes;
use crate::error::{ProcessingError, Result};

// bevy requires an attribute id for each unique vertex attribute. we don't really want to
// expose this to users, so we hash the attribute name to generate a unique id. in theory
// there could be collisions, but in practice this should be fine?
// https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function#FNV-1a_hash
const FNV1A_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x100000001b3;

pub const fn hash_attr_name(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut hash = FNV1A_OFFSET_BASIS;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME);
        i += 1;
    }
    hash
}

#[derive(Component, Clone, Debug, Default)]
pub struct VertexLayout {
    attributes: Vec<Entity>,
}

impl VertexLayout {
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    pub fn with_attributes(attrs: Vec<Entity>) -> Self {
        Self { attributes: attrs }
    }

    pub fn attributes(&self) -> &[Entity] {
        &self.attributes
    }

    pub fn push(&mut self, attr: Entity) {
        if !self.attributes.contains(&attr) {
            self.attributes.push(attr);
        }
    }

    pub fn has_attribute(&self, attr_entity: Entity) -> bool {
        self.attributes.contains(&attr_entity)
    }
}

pub fn create(In(()): In<()>, mut commands: Commands) -> Entity {
    commands.spawn(VertexLayout::new()).id()
}

pub fn create_default(world: &mut World) -> Entity {
    let builtins = world.resource::<BuiltinAttributes>();
    let attrs = vec![
        builtins.position,
        builtins.normal,
        builtins.color,
        builtins.uv,
    ];
    world.spawn(VertexLayout::with_attributes(attrs)).id()
}

pub fn add_position(world: &mut World, entity: Entity) -> Result<()> {
    let position = world.resource::<BuiltinAttributes>().position;
    let mut layout = world
        .get_mut::<VertexLayout>(entity)
        .ok_or(ProcessingError::LayoutNotFound)?;
    layout.push(position);
    Ok(())
}

pub fn add_normal(world: &mut World, entity: Entity) -> Result<()> {
    let normal = world.resource::<BuiltinAttributes>().normal;
    let mut layout = world
        .get_mut::<VertexLayout>(entity)
        .ok_or(ProcessingError::LayoutNotFound)?;
    layout.push(normal);
    Ok(())
}

pub fn add_color(world: &mut World, entity: Entity) -> Result<()> {
    let color = world.resource::<BuiltinAttributes>().color;
    let mut layout = world
        .get_mut::<VertexLayout>(entity)
        .ok_or(ProcessingError::LayoutNotFound)?;
    layout.push(color);
    Ok(())
}

pub fn add_uv(world: &mut World, entity: Entity) -> Result<()> {
    let uv = world.resource::<BuiltinAttributes>().uv;
    let mut layout = world
        .get_mut::<VertexLayout>(entity)
        .ok_or(ProcessingError::LayoutNotFound)?;
    layout.push(uv);
    Ok(())
}

pub fn add_attribute(world: &mut World, layout_entity: Entity, attr_entity: Entity) -> Result<()> {
    let mut layout = world
        .get_mut::<VertexLayout>(layout_entity)
        .ok_or(ProcessingError::LayoutNotFound)?;
    layout.push(attr_entity);
    Ok(())
}

pub fn destroy(In(entity): In<Entity>, mut commands: Commands) {
    commands.entity(entity).despawn();
}
