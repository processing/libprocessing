//! A light in Processing
//!

use bevy::prelude::*;

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn create_directional(
    In((px, py, pz, r, g, b, a, illuminance)): In<(f32, f32, f32, f32, f32, f32, f32, f32)>,
    mut commands: Commands,
) -> Entity {
    commands
        .spawn((
            DirectionalLight {
                illuminance,
                color: Color::srgba(r, g, b, a),
                ..default()
            },
            Transform::from_xyz(px, py, pz),
        ))
        .id()
}

pub fn create_point(
    In((px, py, pz, r, g, b, a, intensity, range, radius)): In<(
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
    )>,
    mut commands: Commands,
) -> Entity {
    commands
        .spawn((
            PointLight {
                intensity,
                color: Color::srgba(r, g, b, a),
                range,
                radius,
                ..default()
            },
            Transform::from_xyz(px, py, pz),
        ))
        .id()
}

pub fn create_spot(
    In((px, py, pz, r, g, b, a, intensity, range, radius, inner_angle, outer_angle)): In<(
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
    )>,
    mut commands: Commands,
) -> Entity {
    commands
        .spawn((
            SpotLight {
                color: Color::srgba(r, g, b, a),
                intensity,
                range,
                radius,
                inner_angle,
                outer_angle,
                ..default()
            },
            Transform::from_xyz(px, py, pz),
        ))
        .id()
}
