//! A light in Processing
//!

use bevy::{camera::visibility::RenderLayers, math::Affine3A, prelude::*};

use crate::{
    error::ProcessingError,
    render::{DrawTransient, RenderState, command::LightKind},
};

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, _app: &mut App) {}
}

/// A light, placed by its `Transform`.
#[derive(Component)]
pub struct ProcessingLight(pub LightKind);

/// The graphics a light belongs to.
#[derive(Component)]
#[relationship(relationship_target = Lights)]
pub struct LightOf(pub Entity);

/// A graphics' lights.
#[derive(Component, Default)]
#[relationship_target(relationship = LightOf, linked_spawn)]
pub struct Lights(Vec<Entity>);

/// Places a light's `Transform` with `matrix`.
pub fn place(matrix: Affine3A, transform: &Transform) -> Transform {
    let position = matrix.transform_point3(transform.translation);
    let forward = matrix.transform_vector3(*transform.forward());
    let up = matrix.transform_vector3(*transform.up());
    Transform::from_translation(position).looking_to(forward, up)
}

/// Spawns the Bevy light for a light, lasting the rest of the graphics' draw.
pub(crate) fn spawn_bevy_light(
    commands: &mut Commands,
    graphics: Entity,
    layers: &RenderLayers,
    kind: LightKind,
    transform: Transform,
) {
    let mut entity = commands.spawn((DrawTransient(graphics), transform, layers.clone()));
    match kind {
        LightKind::Directional { color, illuminance } => {
            entity.insert(DirectionalLight {
                color,
                illuminance,
                ..default()
            });
        }
        LightKind::Point {
            color,
            intensity,
            range,
            radius,
        } => {
            entity.insert(PointLight {
                color,
                intensity,
                range,
                radius,
                ..default()
            });
        }
        LightKind::Spot {
            color,
            intensity,
            range,
            radius,
            inner_angle,
            outer_angle,
        } => {
            entity.insert(SpotLight {
                color,
                intensity,
                range,
                radius,
                inner_angle,
                outer_angle,
                ..default()
            });
        }
    }
}

pub fn create(
    In((graphics, kind, transform)): In<(Entity, LightKind, Transform)>,
    mut commands: Commands,
    graphics_query: Query<(), With<RenderState>>,
) -> Result<Entity, ProcessingError> {
    if !graphics_query.contains(graphics) {
        return Err(ProcessingError::GraphicsNotFound);
    }
    Ok(commands
        .spawn((ProcessingLight(kind), transform, LightOf(graphics)))
        .id())
}

pub fn check_belongs_to(
    In((light, graphics)): In<(Entity, Entity)>,
    lights: Query<&LightOf, With<ProcessingLight>>,
) -> Result<(), ProcessingError> {
    let owner = lights
        .get(light)
        .map_err(|_| ProcessingError::LightNotFound)?;
    if owner.0 != graphics {
        return Err(ProcessingError::LightOnOtherGraphics);
    }
    Ok(())
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    lights: Query<(), With<ProcessingLight>>,
) -> Result<(), ProcessingError> {
    if !lights.contains(entity) {
        return Err(ProcessingError::LightNotFound);
    }
    commands.entity(entity).despawn();
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use bevy::ecs::{relationship::RelationshipTarget, system::RunSystemOnce};

    use super::*;
    use crate::{graphics::begin_draw, render::DrawTransients};

    const DIRECTIONAL: LightKind = LightKind::Directional {
        color: Color::WHITE,
        illuminance: 1000.0,
    };

    const POINT: LightKind = LightKind::Point {
        color: Color::WHITE,
        intensity: 1000.0,
        range: 10.0,
        radius: 0.0,
    };

    fn create_light(world: &mut World, graphics: Entity, kind: LightKind) -> Entity {
        let light = world
            .run_system_once_with(create, (graphics, kind, Transform::default()))
            .unwrap()
            .unwrap();
        world.flush();
        light
    }

    fn spawn_lights(world: &mut World, graphics: Entity, kinds: Vec<LightKind>) -> Vec<Entity> {
        world
            .run_system_once(move |mut commands: Commands| {
                for &kind in &kinds {
                    let layers = RenderLayers::layer(1);
                    spawn_bevy_light(&mut commands, graphics, &layers, kind, Transform::default());
                }
            })
            .unwrap();
        world
            .get::<DrawTransients>(graphics)
            .map(|t| t.iter().collect())
            .unwrap_or_default()
    }

    #[test]
    fn each_light_spawns_a_bevy_light() {
        let mut world = World::new();
        let graphics = world.spawn_empty().id();
        let spawned = spawn_lights(&mut world, graphics, vec![DIRECTIONAL, POINT]);
        assert_eq!(spawned.len(), 2);
        assert!(world.get::<DirectionalLight>(spawned[0]).is_some());
        assert!(world.get::<PointLight>(spawned[1]).is_some());
    }

    #[test]
    fn the_next_draw_clears_spawned_lights() {
        let mut world = World::new();
        let graphics = world.spawn(RenderState::new()).id();
        let spawned = spawn_lights(&mut world, graphics, vec![DIRECTIONAL]);
        world
            .run_system_once_with(begin_draw, graphics)
            .unwrap()
            .unwrap();
        assert!(world.get_entity(spawned[0]).is_err());
    }

    #[test]
    fn a_created_light_is_not_a_bevy_light() {
        let mut world = World::new();
        let graphics = world.spawn(RenderState::new()).id();
        let light = create_light(&mut world, graphics, DIRECTIONAL);
        assert!(world.get::<DirectionalLight>(light).is_none());
    }

    #[test]
    fn lights_are_despawned_with_their_graphics() {
        let mut world = World::new();
        let graphics = world.spawn(RenderState::new()).id();
        let light = create_light(&mut world, graphics, DIRECTIONAL);
        let transform = Transform::from_xyz(1.0, 2.0, 3.0);
        let immediate = world
            .run_system_once_with(create, (graphics, POINT, transform))
            .unwrap()
            .unwrap();
        world.flush();
        assert_eq!(*world.get::<Transform>(immediate).unwrap(), transform);
        world.despawn(graphics);
        assert!(world.get_entity(light).is_err());
        assert!(world.get_entity(immediate).is_err());
    }

    #[test]
    fn a_light_needs_a_graphics() {
        let mut world = World::new();
        let other = world.spawn_empty().id();
        let created = world
            .run_system_once_with(create, (other, POINT, Transform::default()))
            .unwrap();
        assert!(matches!(created, Err(ProcessingError::GraphicsNotFound)));
        let immediate = world
            .run_system_once_with(create, (other, POINT, Transform::default()))
            .unwrap();
        assert!(matches!(immediate, Err(ProcessingError::GraphicsNotFound)));
    }

    #[test]
    fn a_light_only_belongs_to_the_graphics_that_made_it() {
        let mut world = World::new();
        let graphics = world.spawn(RenderState::new()).id();
        let other_graphics = world.spawn(RenderState::new()).id();
        let light = create_light(&mut world, graphics, DIRECTIONAL);
        let own = world
            .run_system_once_with(check_belongs_to, (light, graphics))
            .unwrap();
        assert!(own.is_ok());
        let other = world
            .run_system_once_with(check_belongs_to, (light, other_graphics))
            .unwrap();
        assert!(matches!(other, Err(ProcessingError::LightOnOtherGraphics)));
        let not_a_light = world
            .run_system_once_with(check_belongs_to, (other_graphics, graphics))
            .unwrap();
        assert!(matches!(not_a_light, Err(ProcessingError::LightNotFound)));
    }

    #[test]
    fn destroy_despawns_a_light() {
        let mut world = World::new();
        let graphics = world.spawn(RenderState::new()).id();
        let light = create_light(&mut world, graphics, DIRECTIONAL);
        let result = world.run_system_once_with(destroy, light).unwrap();
        assert!(result.is_ok());
        assert!(world.get_entity(light).is_err());
    }

    #[test]
    fn destroy_rejects_an_entity_that_is_not_a_light() {
        let mut world = World::new();
        let other = world.spawn_empty().id();
        let result = world.run_system_once_with(destroy, other).unwrap();
        assert!(matches!(result, Err(ProcessingError::LightNotFound)));
        assert!(world.get_entity(other).is_ok());
    }

    #[test]
    fn identity_keeps_the_placement() {
        let transform = Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y);
        let placed = place(Affine3A::IDENTITY, &transform);
        assert!(placed.translation.abs_diff_eq(transform.translation, 1e-5));
        assert!(placed.rotation.abs_diff_eq(transform.rotation, 1e-5));
    }

    #[test]
    fn current_matrix_moves_the_light() {
        let matrix = Affine3A::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let placed = place(matrix, &Transform::from_xyz(0.0, 5.0, 0.0));
        assert_eq!(placed.translation, Vec3::new(10.0, 5.0, 0.0));
    }

    #[test]
    fn current_matrix_rotates_the_light_without_scaling_it() {
        let matrix = Affine3A::from_scale(Vec3::splat(2.0)) * Affine3A::from_rotation_y(FRAC_PI_2);
        let placed = place(matrix, &Transform::default());
        assert!(placed.scale.abs_diff_eq(Vec3::ONE, 1e-5));
        assert!(placed.forward().dot(Vec3::NEG_X) > 0.999);
    }
}
