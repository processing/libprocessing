//! A light in Processing
//!

use bevy::{camera::visibility::RenderLayers, prelude::*};

use crate::{error::ProcessingError, graphics::Graphics};

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn create_directional(
    In((entity, px, py, pz, color, illuminance)): In<(Entity, f32, f32, f32, Color, f32)>,
    mut commands: Commands,
    graphics: Query<&RenderLayers, With<Graphics>>,
) -> Result<Entity, ProcessingError> {
    let layer = graphics
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;
    Ok(commands
        .spawn((
            DirectionalLight {
                illuminance,
                color,
                ..default()
            },
            Transform::from_xyz(px, py, pz),
            layer.clone(),
        ))
        .id())
}

pub fn create_point(
    In((entity, px, py, pz, color, intensity, range, radius)): In<(
        Entity,
        f32,
        f32,
        f32,
        Color,
        f32,
        f32,
        f32,
    )>,
    mut commands: Commands,
    graphics: Query<&RenderLayers, With<Graphics>>,
) -> Result<Entity, ProcessingError> {
    let layer = graphics
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;
    Ok(commands
        .spawn((
            PointLight {
                intensity,
                color,
                range,
                radius,
                ..default()
            },
            Transform::from_xyz(px, py, pz),
            layer.clone(),
        ))
        .id())
}

pub fn create_spot(
    In((entity, px, py, pz, color, intensity, range, radius, inner_angle, outer_angle)): In<(
        Entity,
        f32,
        f32,
        f32,
        Color,
        f32,
        f32,
        f32,
        f32,
        f32,
    )>,
    mut commands: Commands,
    graphics: Query<&RenderLayers, With<Graphics>>,
) -> Result<Entity, ProcessingError> {
    let layer = graphics
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;
    Ok(commands
        .spawn((
            SpotLight {
                color,
                intensity,
                range,
                radius,
                inner_angle,
                outer_angle,
                ..default()
            },
            Transform::from_xyz(px, py, pz),
            layer.clone(),
        ))
        .id())
}
