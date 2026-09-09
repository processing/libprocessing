//! Rigid-body physics (avian3d).
//!
//! Bodies are pure simulation objects: create them, step happens once per
//! frame (frame-locked, deterministic), read poses back and draw them however
//! the sketch likes — typically in bulk into a particle system's
//! position/rotation buffers for instanced drawing.

use bevy::math::{Quat, Vec3};
use bevy::prelude::Entity;
use processing_physics::{
    body_apply_angular_impulse, body_apply_impulse, body_create, body_create_from_geometry,
    body_destroy,
    body_get_linear_velocity, body_get_transform, body_set_angular_velocity, body_set_density,
    body_set_friction, body_set_linear_velocity, body_set_position, body_set_restitution,
    body_set_transform, read_transforms, set_gravity, set_paused, set_realtime, set_substeps,
    set_timestep_hz, step,
};

use crate::error;

// ── World configuration ─────────────────────────────────────────────────────

/// Set the gravity vector (world units per second squared).
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_set_gravity(x: f32, y: f32, z: f32) {
    error::clear_error();
    error::check(|| set_gravity(Vec3::new(x, y, z)));
}

/// Frame-locked stepping: the simulation advances by exactly `1/hz` seconds
/// every rendered frame (the default, at 60 Hz). Deterministic.
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_set_timestep(hz: f64) {
    error::clear_error();
    error::check(|| set_timestep_hz(hz));
}

/// Follow the wall clock instead of frame-locked stepping (non-deterministic).
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_set_realtime() {
    error::clear_error();
    error::check(set_realtime);
}

/// Mark the start of a Processing frame: the next engine update advances the
/// simulation by one fixed step. Called once per frame by the host renderer
/// (primary canvas beginDraw); extra engine updates pumped by flushes or
/// offscreen graphics within the frame do not step again.
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_step() {
    error::clear_error();
    error::check(step);
}

/// Pause or resume the simulation.
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_set_paused(paused: bool) {
    error::clear_error();
    error::check(|| set_paused(paused));
}

/// Solver substeps per physics step (default 6). More = stiffer stacks.
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_set_substeps(count: u32) {
    error::clear_error();
    error::check(|| set_substeps(count));
}

// ── Bodies ──────────────────────────────────────────────────────────────────

/// Create a rigid body.
///
/// `kind`: 0 = static, 1 = dynamic, 2 = kinematic.
/// `shape`: 0 = box (p0, p1, p2 = full extents), 1 = sphere (p0 = radius),
/// 2 = capsule (p0 = radius, p1 = cylinder length).
/// Returns the body id, or 0 on error.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn processing_physics_body_create(
    kind: u32,
    shape: u32,
    p0: f32,
    p1: f32,
    p2: f32,
    x: f32,
    y: f32,
    z: f32,
    qx: f32,
    qy: f32,
    qz: f32,
    qw: f32,
) -> u64 {
    error::clear_error();
    error::check(|| {
        body_create(
            kind,
            shape,
            Vec3::new(p0, p1, p2),
            Vec3::new(x, y, z),
            Quat::from_xyzw(qx, qy, qz, qw).normalize(),
        )
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

/// Create a rigid body whose collider is built from a geometry's CPU mesh
/// (any geometry id: primitives, custom shapes, glTF meshes).
///
/// `kind`: 0 = static, 1 = dynamic, 2 = kinematic.
/// `mode`: 0 = trimesh (exact; static scenery), 1 = convex hull (fast;
/// dynamic bodies), 2 = convex decomposition (accurate and dynamic-friendly,
/// expensive to build).
/// Returns the body id, or 0 on error.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn processing_physics_body_create_from_geometry(
    geometry_id: u64,
    kind: u32,
    mode: u32,
    x: f32,
    y: f32,
    z: f32,
    qx: f32,
    qy: f32,
    qz: f32,
    qw: f32,
) -> u64 {
    error::clear_error();
    error::check(|| {
        body_create_from_geometry(
            Entity::from_bits(geometry_id),
            kind,
            mode,
            Vec3::new(x, y, z),
            Quat::from_xyzw(qx, qy, qz, qw).normalize(),
        )
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

/// Remove a body from the simulation.
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_body_destroy(body_id: u64) {
    error::clear_error();
    error::check(|| body_destroy(Entity::from_bits(body_id)));
}

/// Teleport a body (also zeroes accumulated interpolation, not velocities).
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn processing_physics_body_set_transform(
    body_id: u64,
    x: f32,
    y: f32,
    z: f32,
    qx: f32,
    qy: f32,
    qz: f32,
    qw: f32,
) {
    error::clear_error();
    error::check(|| {
        body_set_transform(
            Entity::from_bits(body_id),
            Vec3::new(x, y, z),
            Quat::from_xyzw(qx, qy, qz, qw).normalize(),
        )
    });
}

/// Position-only teleport: keeps the current rotation and velocities.
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_body_set_position(body_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    error::check(|| body_set_position(Entity::from_bits(body_id), Vec3::new(x, y, z)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_body_set_linear_velocity(body_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    error::check(|| body_set_linear_velocity(Entity::from_bits(body_id), Vec3::new(x, y, z)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_body_set_angular_velocity(
    body_id: u64,
    x: f32,
    y: f32,
    z: f32,
) {
    error::clear_error();
    error::check(|| body_set_angular_velocity(Entity::from_bits(body_id), Vec3::new(x, y, z)));
}

/// Apply a one-shot linear impulse (mass-scaled velocity change) this frame.
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_body_apply_impulse(body_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    error::check(|| body_apply_impulse(Entity::from_bits(body_id), Vec3::new(x, y, z)));
}

/// Apply a one-shot angular impulse this frame.
#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_body_apply_angular_impulse(
    body_id: u64,
    x: f32,
    y: f32,
    z: f32,
) {
    error::clear_error();
    error::check(|| body_apply_angular_impulse(Entity::from_bits(body_id), Vec3::new(x, y, z)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_body_set_friction(body_id: u64, friction: f32) {
    error::clear_error();
    error::check(|| body_set_friction(Entity::from_bits(body_id), friction));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_body_set_restitution(body_id: u64, restitution: f32) {
    error::clear_error();
    error::check(|| body_set_restitution(Entity::from_bits(body_id), restitution));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_physics_body_set_density(body_id: u64, density: f32) {
    error::clear_error();
    error::check(|| body_set_density(Entity::from_bits(body_id), density));
}

// ── Readback ────────────────────────────────────────────────────────────────

/// Write a body's pose into `out` as 7 floats: position xyz, quaternion xyzw.
///
/// SAFETY:
/// - Init has been called.
/// - `out` is valid for 7 f32 writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_physics_body_get_transform(body_id: u64, out: *mut f32) {
    error::clear_error();
    if let Some(pose) = error::check(|| body_get_transform(Entity::from_bits(body_id))) {
        let out = unsafe { std::slice::from_raw_parts_mut(out, 7) };
        out.copy_from_slice(&pose);
    }
}

/// Write a body's linear velocity into `out` as 3 floats.
///
/// SAFETY:
/// - Init has been called.
/// - `out` is valid for 3 f32 writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_physics_body_get_linear_velocity(body_id: u64, out: *mut f32) {
    error::clear_error();
    if let Some(v) = error::check(|| body_get_linear_velocity(Entity::from_bits(body_id))) {
        let out = unsafe { std::slice::from_raw_parts_mut(out, 3) };
        out.copy_from_slice(&v);
    }
}

/// Bulk pose readback: for each of `count` body ids, writes 7 floats
/// (position xyz, quaternion xyzw) into `out`. Destroyed bodies write
/// identity poses. Returns the number of live bodies read.
///
/// SAFETY:
/// - Init has been called.
/// - `ids` is valid for `count` u64 reads; `out` for `count * 7` f32 writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_physics_read_transforms(
    ids: *const u64,
    count: usize,
    out: *mut f32,
) -> usize {
    error::clear_error();
    let ids = unsafe { std::slice::from_raw_parts(ids, count) };
    let out = unsafe { std::slice::from_raw_parts_mut(out, count * 7) };
    match error::check(|| {
        let mut poses = vec![0.0f32; count * 7];
        let read = read_transforms(ids, &mut poses)?;
        Ok((read, poses))
    }) {
        Some((read, poses)) => {
            out.copy_from_slice(&poses);
            read
        }
        None => 0,
    }
}
