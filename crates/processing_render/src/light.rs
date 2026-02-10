//! A light in Processing
//!

use bevy::{camera::visibility::RenderLayers, prelude::*};

use crate::{error::ProcessingError, graphics::Graphics};

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn create_directional(
    In((entity, color, illuminance)): In<(Entity, Color, f32)>,
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
            layer.clone(),
        ))
        .id())
}

pub fn create_point(
    In((entity, color, intensity, range, radius)): In<(Entity, Color, f32, f32, f32)>,
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
            layer.clone(),
        ))
        .id())
}

pub fn create_spot(
    In((entity, color, intensity, range, radius, inner_angle, outer_angle)): In<(
        Entity,
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
            layer.clone(),
        ))
        .id())
}
