pub mod pbr;

use bevy::prelude::*;

use crate::error::{ProcessingError, Result};
use crate::render::material::UntypedMaterial;

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        let handle = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                unlit: true,
                cull_mode: None,
                base_color: Color::WHITE,
                ..default()
            });
        let entity = world.spawn(UntypedMaterial(handle.untyped())).id();
        world.insert_resource(DefaultMaterial(entity));
    }
}

#[derive(Resource)]
pub struct DefaultMaterial(pub Entity);

#[derive(Debug, Clone)]
pub enum MaterialValue {
    Float(f32),
    Float2([f32; 2]),
    Float3([f32; 3]),
    Float4([f32; 4]),
    Int(i32),
    Int2([i32; 2]),
    Int3([i32; 3]),
    Int4([i32; 4]),
    UInt(u32),
    Mat4([f32; 16]),
    Texture(Entity),
}

pub fn create_pbr(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let handle = materials.add(StandardMaterial {
        unlit: false,
        cull_mode: None,
        ..default()
    });
    commands.spawn(UntypedMaterial(handle.untyped())).id()
}

pub fn set_property(
    In((entity, name, value)): In<(Entity, String, MaterialValue)>,
    material_handles: Query<&UntypedMaterial>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) -> Result<()> {
    let untyped = material_handles
        .get(entity)
        .map_err(|_| ProcessingError::MaterialNotFound)?;
    let handle = untyped
        .0
        .clone()
        .try_typed::<StandardMaterial>()
        .map_err(|_| ProcessingError::MaterialNotFound)?;
    let standard = standard_materials
        .get_mut(&handle)
        .ok_or(ProcessingError::MaterialNotFound)?;
    pbr::set_property(standard, &name, &value)?;
    Ok(())
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    material_handles: Query<&UntypedMaterial>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) -> Result<()> {
    let untyped = material_handles
        .get(entity)
        .map_err(|_| ProcessingError::MaterialNotFound)?;
    if let Ok(handle) = untyped.0.clone().try_typed::<StandardMaterial>() {
        standard_materials.remove(&handle);
    }
    commands.entity(entity).despawn();
    Ok(())
}
