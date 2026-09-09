//! A light in Processing
//!

use bevy::{camera::visibility::RenderLayers, prelude::*};

use crate::{error::ProcessingError, graphics::Graphics};

/// Calibration: the engine's light contract is PHYSICAL LUX at the default
/// camera exposure (EV100 9.7, Blender's default — bevy's own convention).
///
/// The current bevy-tib-rebase vintage renders analytical (directional/point/
/// spot) lights ~1/exposure(9.7)^2 = 9.2e5x hotter than physical: measured on
/// the grey-card probe (quil `dev/greycard_once.clj`, linear-pipeline fits
/// kappa = 7.5e5..9.6e5 across half a decade of lux), while ambient light,
/// emissive, and the camera `Exposure` knob itself all behave correctly (the
/// exposure law verified exact over 10 EV of camera exposure). Scaling
/// illuminance/intensity by exposure(9.7)^2 at creation restores the physical
/// contract without touching the (correct) ambient and emissive paths.
/// Re-verify with the probe after any bevy rebase; if upstream fixes its
/// light scaling this constant must be re-measured or the scene goes dark.
const LIGHT_EXPOSURE_COMPENSATION: f32 = 1.0828123e-6; // (exp2(-9.7) / 1.2)^2

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, _app: &mut App) {}
}

/// Despawn a light entity (both retained-API and immediate-mode lights).
pub fn destroy(In(entity): In<Entity>, mut commands: Commands) -> Result<(), ProcessingError> {
    commands.entity(entity).despawn();
    Ok(())
}

/// Aim a light: `dir` is the direction the light TRAVELS (Processing's
/// directionalLight convention). Distinct from the generic Euler-angle
/// transform setter, which cannot express "point this way".
pub fn set_direction(
    In((entity, dir)): In<(Entity, Vec3)>,
    mut transforms: Query<&mut Transform>,
) -> Result<(), ProcessingError> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::InvalidEntity)?;
    let dir = dir.try_normalize().ok_or(ProcessingError::InvalidArgument(
        "light direction must be non-zero".into(),
    ))?;
    // Lights shine along their -Z; look_to aims -Z at `dir`.
    let up = if dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
    transform.look_to(dir, up);
    Ok(())
}

/// Toggle shadow casting on a directional light.
pub fn set_shadows(
    In((entity, enabled)): In<(Entity, bool)>,
    mut lights: Query<&mut DirectionalLight>,
) -> Result<(), ProcessingError> {
    let mut light = lights
        .get_mut(entity)
        .map_err(|_| ProcessingError::InvalidEntity)?;
    light.shadow_maps_enabled = enabled;
    Ok(())
}

/// Toggle contact-shadow casting on a directional light.
///
/// Separate from `set_shadows`: shadow maps and contact shadows are
/// independent gates on the same light, and the effect also has to be enabled
/// on the camera (`graphics::set_contact_shadows`) before a light's flag does
/// anything.
pub fn set_contact_shadows(
    In((entity, enabled)): In<(Entity, bool)>,
    mut lights: Query<&mut DirectionalLight>,
) -> Result<(), ProcessingError> {
    let mut light = lights
        .get_mut(entity)
        .map_err(|_| ProcessingError::InvalidEntity)?;
    light.contact_shadows_enabled = enabled;
    Ok(())
}

/// Set the shadow-map depth and normal bias for a directional light.
///
/// These are the two knobs for shadow acne -- the self-shadowing stipple a
/// surface gets when its own depth in the shadow map lands ambiguously.
/// Raising them trades acne for peter-panning (shadows detaching from their
/// caster). A stack of coplanar faces is the scene that needs them.
pub fn set_shadow_bias(
    In((entity, depth_bias, normal_bias)): In<(Entity, f32, f32)>,
    mut lights: Query<&mut DirectionalLight>,
) -> Result<(), ProcessingError> {
    let mut light = lights
        .get_mut(entity)
        .map_err(|_| ProcessingError::InvalidEntity)?;
    light.shadow_depth_bias = depth_bias;
    light.shadow_normal_bias = normal_bias;
    Ok(())
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
                illuminance: illuminance * LIGHT_EXPOSURE_COMPENSATION,
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
                intensity: intensity * LIGHT_EXPOSURE_COMPENSATION,
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
                intensity: intensity * LIGHT_EXPOSURE_COMPENSATION,
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
