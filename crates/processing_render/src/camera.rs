//! Supplies the following camera controllers:
//! - Orbit camera: left mouse button to orbit, middle mouse button to pan, right mouse button or
//! scroll wheel to zoom. Inspired by [PeasyCam](https://github.com/jdf/peasycam) by Jonathan
//! Feinberg.
//! - Free camera: WASD to move, mouse to look around.
//! - Pan camera: middle mouse button to pan, scroll wheel to zoom.
use std::f32::consts::FRAC_PI_2;

use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraState};
use bevy::camera_controller::pan_camera::PanCamera;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton};
use bevy::prelude::*;

#[derive(Clone, Copy, Default)]
pub enum RotationMode {
    #[default]
    SuppressRoll,
    YawOnly,
}

#[derive(Component)]
pub struct OrbitCamera {
    pub center: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub orbit_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub rotation_mode: RotationMode,
    pub initial_center: Vec3,
    pub initial_distance: f32,
    pub initial_yaw: f32,
    pub initial_pitch: f32,
}

impl OrbitCamera {
    pub fn new(center: Vec3, distance: f32) -> Self {
        Self {
            center,
            distance,
            yaw: 0.0,
            pitch: 0.0,
            min_distance: 1.0,
            max_distance: f32::MAX,
            orbit_sensitivity: 0.005,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 0.1,
            rotation_mode: RotationMode::default(),
            initial_center: center,
            initial_distance: distance,
            initial_yaw: 0.0,
            initial_pitch: 0.0,
        }
    }

    fn rotation(&self) -> Quat {
        Quat::from_rotation_y(-self.yaw) * Quat::from_rotation_x(-self.pitch)
    }

    fn apply_to_transform(&self, transform: &mut Transform) {
        let rotation = self.rotation();
        transform.translation = self.center + rotation * Vec3::new(0.0, 0.0, self.distance);
        transform.look_at(self.center, Vec3::Y);
    }

    pub fn reset(&mut self) {
        self.center = self.initial_center;
        self.distance = self.initial_distance;
        self.yaw = self.initial_yaw;
        self.pitch = self.initial_pitch;
    }
}

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RunFixedMainLoop,
            update_orbit_camera.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        );
    }
}

fn update_orbit_camera(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let left = mouse_buttons.pressed(MouseButton::Left);
    let middle = mouse_buttons.pressed(MouseButton::Middle);
    let right = mouse_buttons.pressed(MouseButton::Right);
    let delta = mouse_motion.delta;
    let scroll = mouse_scroll.delta.y;

    for (mut transform, mut orbit) in query.iter_mut() {
        let mut changed = false;

        if left && delta != Vec2::ZERO {
            orbit.yaw += delta.x * orbit.orbit_sensitivity;
            if matches!(orbit.rotation_mode, RotationMode::SuppressRoll) {
                let limit = FRAC_PI_2 - 0.01;
                orbit.pitch =
                    (orbit.pitch + delta.y * orbit.orbit_sensitivity).clamp(-limit, limit);
            }
            changed = true;
        }

        if middle && delta != Vec2::ZERO {
            let pan_speed = orbit.pan_sensitivity * orbit.distance * 0.001;
            orbit.center -= transform.right() * delta.x * pan_speed;
            orbit.center += transform.up() * delta.y * pan_speed;
            changed = true;
        }

        if right && delta != Vec2::ZERO {
            orbit.distance *= 1.0 + delta.y * orbit.zoom_sensitivity;
            orbit.distance = orbit.distance.clamp(orbit.min_distance, orbit.max_distance);
            changed = true;
        }

        if scroll != 0.0 {
            orbit.distance *= 1.0 - scroll * orbit.zoom_sensitivity;
            orbit.distance = orbit.distance.clamp(orbit.min_distance, orbit.max_distance);
            changed = true;
        }

        if changed {
            orbit.apply_to_transform(&mut transform);
        }
    }
}

/// Enables an orbit camera on the specified entity.
pub fn enable_orbit_camera(
    In(entity): In<Entity>,
    mut commands: Commands,
    transforms: Query<&Transform>,
    sizes: Query<&crate::graphics::SurfaceSize>,
) -> crate::error::Result<()> {
    let center = Vec3::ZERO;
    let distance = if let Ok(transform) = transforms.get(entity) {
        transform.translation.distance(center)
    } else if let Ok(crate::graphics::SurfaceSize(_, height)) = sizes.get(entity) {
        let fov = std::f32::consts::PI / 3.0;
        (*height as f32 / 2.0) / (fov / 2.0).tan()
    } else {
        400.0
    };

    commands
        .entity(entity)
        .remove::<FreeCamera>()
        .remove::<PanCamera>()
        .insert(OrbitCamera::new(center, distance));
    Ok(())
}

/// Enables a free camera on the specified entity.
pub fn enable_free_camera(
    In(entity): In<Entity>,
    mut commands: Commands,
    sizes: Query<&crate::graphics::SurfaceSize>,
) -> crate::error::Result<()> {
    // processing uses pixel-scale coordinates
    let speed = if let Ok(crate::graphics::SurfaceSize(_, height)) = sizes.get(entity) {
        *height as f32
    } else {
        400.0
    };

    commands
        .entity(entity)
        .remove::<OrbitCamera>()
        .remove::<PanCamera>()
        .insert(FreeCamera {
            walk_speed: speed,
            run_speed: speed * 3.0,
            ..default()
        });
    Ok(())
}

/// Enables a pan camera on the specified entity.
pub fn enable_pan_camera(
    In(entity): In<Entity>,
    mut commands: Commands,
) -> crate::error::Result<()> {
    commands
        .entity(entity)
        .remove::<OrbitCamera>()
        .remove::<FreeCamera>()
        .insert(PanCamera::default());
    Ok(())
}

/// Disables all camera controllers on the specified entity.
pub fn disable_camera_controller(
    In(entity): In<Entity>,
    mut commands: Commands,
) -> crate::error::Result<()> {
    commands
        .entity(entity)
        .remove::<OrbitCamera>()
        .remove::<FreeCamera>()
        .remove::<PanCamera>();
    Ok(())
}

pub fn set_distance(
    In((entity, distance)): In<(Entity, f32)>,
    mut orbits: Query<(&mut Transform, &mut OrbitCamera)>,
    mut pans: Query<&mut PanCamera>,
) -> crate::error::Result<()> {
    if let Ok((mut transform, mut orbit)) = orbits.get_mut(entity) {
        orbit.distance = distance.clamp(orbit.min_distance, orbit.max_distance);
        orbit.apply_to_transform(&mut transform);
    } else if let Ok(mut pan) = pans.get_mut(entity) {
        pan.zoom_factor = distance.clamp(pan.min_zoom, pan.max_zoom);
    }
    Ok(())
}

pub fn set_center(
    In((entity, center)): In<(Entity, Vec3)>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
) -> crate::error::Result<()> {
    if let Ok((mut transform, mut orbit)) = query.get_mut(entity) {
        orbit.center = center;
        orbit.apply_to_transform(&mut transform);
    }
    Ok(())
}

pub fn set_min_distance(
    In((entity, min)): In<(Entity, f32)>,
    mut orbits: Query<&mut OrbitCamera>,
    mut pans: Query<&mut PanCamera>,
) -> crate::error::Result<()> {
    if let Ok(mut orbit) = orbits.get_mut(entity) {
        orbit.min_distance = min;
    } else if let Ok(mut pan) = pans.get_mut(entity) {
        pan.min_zoom = min;
    }
    Ok(())
}

pub fn set_max_distance(
    In((entity, max)): In<(Entity, f32)>,
    mut orbits: Query<&mut OrbitCamera>,
    mut pans: Query<&mut PanCamera>,
) -> crate::error::Result<()> {
    if let Ok(mut orbit) = orbits.get_mut(entity) {
        orbit.max_distance = max;
    } else if let Ok(mut pan) = pans.get_mut(entity) {
        pan.max_zoom = max;
    }
    Ok(())
}

pub fn set_speed(
    In((entity, speed)): In<(Entity, f32)>,
    mut orbits: Query<&mut OrbitCamera>,
    mut frees: Query<&mut FreeCamera>,
    mut pans: Query<&mut PanCamera>,
) -> crate::error::Result<()> {
    if let Ok(mut orbit) = orbits.get_mut(entity) {
        orbit.orbit_sensitivity = speed;
    } else if let Ok(mut free) = frees.get_mut(entity) {
        free.walk_speed = speed;
        free.run_speed = speed * 3.0;
    } else if let Ok(mut pan) = pans.get_mut(entity) {
        pan.pan_speed = speed;
    }
    Ok(())
}

pub fn reset_camera(
    In(entity): In<Entity>,
    mut orbits: Query<(&mut Transform, &mut OrbitCamera)>,
    mut free_states: Query<&mut FreeCameraState>,
    mut pans: Query<&mut PanCamera>,
) -> crate::error::Result<()> {
    if let Ok((mut transform, mut orbit)) = orbits.get_mut(entity) {
        orbit.reset();
        orbit.apply_to_transform(&mut transform);
    } else if let Ok(mut state) = free_states.get_mut(entity) {
        state.pitch = 0.0;
        state.yaw = 0.0;
        state.velocity = Vec3::ZERO;
        state.speed_multiplier = 1.0;
    } else if let Ok(mut pan) = pans.get_mut(entity) {
        pan.zoom_factor = 1.0;
    }
    Ok(())
}
