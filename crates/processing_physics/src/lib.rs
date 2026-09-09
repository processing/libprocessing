//! Physics integration for processing, wrapping the avian3d physics engine.
//!
//! Bodies are pure simulation entities: they carry no render components. A
//! sketch draws them itself by reading transforms back (typically in bulk into
//! a particle system's position/rotation buffers for instanced drawing).
//!
//! Stepping is frame-locked by default: `Time<Physics>` is paused and advanced
//! by exactly `1/hz` every app update, so a sketch rendered frame-by-frame is
//! deterministic regardless of wall-clock jitter (same policy as the paused
//! animation-pool puppets). `set_realtime` opts back into wall-clock stepping.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use processing_core::app_mut;
use processing_core::error::Result;

pub use avian3d;

use crate::error::PhysicsError;

pub mod error {
    use processing_core::error::ProcessingError;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum PhysicsError {
        #[error("unknown rigid body kind {0} (expected 0=static, 1=dynamic, 2=kinematic)")]
        UnknownBodyKind(u32),
        #[error("unknown collider shape {0} (expected 0=box, 1=sphere, 2=capsule)")]
        UnknownShape(u32),
        #[error(
            "unknown mesh collider mode {0} (expected 0=trimesh, 1=convex hull, 2=convex decomposition)"
        )]
        UnknownMeshMode(u32),
        #[error("collider construction from mesh failed (empty or non-triangle geometry?)")]
        MeshColliderFailed,
        #[error("entity is not a physics body")]
        NotABody,
    }

    impl From<PhysicsError> for ProcessingError {
        fn from(err: PhysicsError) -> Self {
            ProcessingError::InvalidArgument(err.to_string())
        }
    }
}

/// How the physics clock advances each app update.
#[derive(Resource, Debug, Clone, Copy)]
pub struct PhysicsFrameClock {
    /// Fixed timestep frequency; each rendered frame advances the simulation
    /// by exactly `1/hz` seconds.
    pub hz: f64,
    /// When false, `Time<Physics>` is unpaused and follows the wall clock.
    pub frame_locked: bool,
    /// When true, the frame clock stops advancing (bodies freeze).
    pub paused: bool,
    /// One step is owed. Set by [`step`] at the start of a Processing frame
    /// and consumed by the first app update that follows, so extra updates
    /// pumped by flushes/offscreen graphics within the same frame don't
    /// advance the simulation again.
    pub pending: bool,
}

impl Default for PhysicsFrameClock {
    fn default() -> Self {
        Self {
            hz: 60.0,
            frame_locked: true,
            paused: false,
            pending: false,
        }
    }
}

/// Marker for entities created through the processing physics API.
#[derive(Component)]
pub struct PhysicsBody;

pub struct ProcessingPhysicsPlugin;

impl Plugin for ProcessingPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::new(PostUpdate));
        app.init_resource::<PhysicsFrameClock>();
        app.add_systems(Startup, pause_physics_clock);
        app.add_systems(
            PostUpdate,
            advance_frame_clock.before(PhysicsSystems::First),
        );
    }
}

fn pause_physics_clock(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

/// Frame-locked stepping: while `Time<Physics>` is paused, avian's schedule
/// runner uses whatever delta a manual `advance_by` set, so advancing by
/// exactly `1/hz` here steps the simulation deterministically per frame.
/// Gated on `pending` — the host marks each Processing frame with [`step`],
/// and every flush-pumped app update in between is a no-op.
fn advance_frame_clock(mut clock: ResMut<PhysicsFrameClock>, mut time: ResMut<Time<Physics>>) {
    if clock.frame_locked {
        if !time.is_paused() {
            time.pause();
        }
        if clock.pending && !clock.paused {
            time.advance_by(Duration::from_secs_f64(1.0 / clock.hz));
        }
        clock.pending = false;
    } else if time.is_paused() && !clock.paused {
        time.unpause();
    } else if clock.paused && !time.is_paused() {
        time.pause();
    }
}

/// Mark the start of a Processing frame: the next app update advances the
/// simulation by one fixed step (frame-locked mode only; a wall-clock world
/// steps on every update regardless).
pub fn step() -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .resource_mut::<PhysicsFrameClock>()
            .pending = true;
        Ok(())
    })
}

// ── World configuration ─────────────────────────────────────────────────────

pub fn set_gravity(gravity: Vec3) -> Result<()> {
    app_mut(|app| {
        app.world_mut().insert_resource(Gravity(gravity));
        Ok(())
    })
}

pub fn set_timestep_hz(hz: f64) -> Result<()> {
    app_mut(|app| {
        let mut clock = app.world_mut().resource_mut::<PhysicsFrameClock>();
        clock.hz = hz.max(1.0);
        clock.frame_locked = true;
        Ok(())
    })
}

pub fn set_realtime() -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .resource_mut::<PhysicsFrameClock>()
            .frame_locked = false;
        Ok(())
    })
}

pub fn set_paused(paused: bool) -> Result<()> {
    app_mut(|app| {
        app.world_mut().resource_mut::<PhysicsFrameClock>().paused = paused;
        Ok(())
    })
}

pub fn set_substeps(count: u32) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .insert_resource(SubstepCount(count.max(1)));
        Ok(())
    })
}

// ── Bodies ──────────────────────────────────────────────────────────────────

fn body_kind(kind: u32) -> std::result::Result<RigidBody, PhysicsError> {
    match kind {
        0 => Ok(RigidBody::Static),
        1 => Ok(RigidBody::Dynamic),
        2 => Ok(RigidBody::Kinematic),
        other => Err(PhysicsError::UnknownBodyKind(other)),
    }
}

/// Shape params: box = full extents (x, y, z); sphere = (radius, _, _);
/// capsule = (radius, length, _) where length is the cylindrical section.
fn collider_shape(shape: u32, p: Vec3) -> std::result::Result<Collider, PhysicsError> {
    match shape {
        0 => Ok(Collider::cuboid(p.x, p.y, p.z)),
        1 => Ok(Collider::sphere(p.x)),
        2 => Ok(Collider::capsule(p.x, p.y)),
        other => Err(PhysicsError::UnknownShape(other)),
    }
}

pub fn body_create(
    kind: u32,
    shape: u32,
    params: Vec3,
    position: Vec3,
    rotation: Quat,
) -> Result<Entity> {
    app_mut(|app| {
        let body = body_kind(kind)?;
        let collider = collider_shape(shape, params)?;
        Ok(app
            .world_mut()
            .spawn((
                PhysicsBody,
                body,
                collider,
                Transform::from_translation(position).with_rotation(rotation),
            ))
            .id())
    })
}

/// Create a rigid body whose collider is built from an existing geometry's
/// CPU mesh (the same entities the geometry/glTF APIs hand out).
///
/// `mode`: 0 = trimesh (exact, best for static scenery), 1 = convex hull
/// (fast, best for dynamic bodies), 2 = convex decomposition (accurate AND
/// dynamic-friendly, expensive to build).
pub fn body_create_from_geometry(
    geometry: Entity,
    kind: u32,
    mode: u32,
    position: Vec3,
    rotation: Quat,
) -> Result<Entity> {
    app_mut(|app| {
        let body = body_kind(kind)?;
        let world = app.world_mut();
        let handle = world
            .get::<processing_render::geometry::Geometry>(geometry)
            .ok_or(processing_core::error::ProcessingError::GeometryNotFound)?
            .handle
            .clone();
        let meshes = world.resource::<Assets<Mesh>>();
        let mesh = meshes.get(&handle).ok_or(processing_core::error::ProcessingError::GeometryNotFound)?;
        let collider = match mode {
            0 => Collider::trimesh_from_mesh(mesh),
            1 => Collider::convex_hull_from_mesh(mesh),
            2 => Collider::convex_decomposition_from_mesh(mesh),
            other => return Err(PhysicsError::UnknownMeshMode(other).into()),
        }
        // Engine meshes (glTF in particular) are often non-indexed triangle
        // soups, which avian's *_from_mesh helpers reject; sequential indices
        // are exact for a soup, so rebuild from raw positions.
        .or_else(|| collider_from_triangle_soup(mesh, mode))
        .ok_or_else(|| {
            processing_core::error::ProcessingError::InvalidArgument(format!(
                "collider construction from mesh failed (position attr: {:?}, indices: {:?}, topology: {:?})",
                mesh.attribute(Mesh::ATTRIBUTE_POSITION).map(|a| {
                    format!(
                        "{} verts, float32x3: {}",
                        a.len(),
                        matches!(a, bevy::mesh::VertexAttributeValues::Float32x3(_))
                    )
                }),
                mesh.indices().map(|i| i.len()),
                mesh.primitive_topology(),
            ))
        })?;
        Ok(world
            .spawn((
                PhysicsBody,
                body,
                collider,
                Transform::from_translation(position).with_rotation(rotation),
            ))
            .id())
    })
}

/// Collider from a non-indexed TriangleList mesh: consecutive vertex triples
/// are the triangles, so sequential indices reconstruct the topology exactly.
fn collider_from_triangle_soup(mesh: &Mesh, mode: u32) -> Option<Collider> {
    use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
    if mesh.primitive_topology() != PrimitiveTopology::TriangleList || mesh.indices().is_some() {
        return None;
    }
    let VertexAttributeValues::Float32x3(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?
    else {
        return None;
    };
    let vertices: Vec<Vec3> = positions.iter().map(|p| Vec3::from_array(*p)).collect();
    if vertices.len() < 3 {
        return None;
    }
    let indices: Vec<[u32; 3]> = (0..vertices.len() as u32 / 3)
        .map(|t| [t * 3, t * 3 + 1, t * 3 + 2])
        .collect();
    match mode {
        0 => Some(Collider::trimesh(vertices, indices)),
        1 => Collider::convex_hull(vertices),
        2 => Some(Collider::convex_decomposition(vertices, indices)),
        _ => None,
    }
}

pub fn body_destroy(entity: Entity) -> Result<()> {
    app_mut(|app| {
        app.world_mut().entity_mut(entity).despawn();
        Ok(())
    })
}

/// Teleport. Writes both `Transform` and avian's `Position`/`Rotation` so the
/// change lands regardless of where in the frame it happens.
pub fn body_set_transform(entity: Entity, position: Vec3, rotation: Quat) -> Result<()> {
    app_mut(|app| {
        let mut e = app.world_mut().entity_mut(entity);
        let mut transform = e.get_mut::<Transform>().ok_or(PhysicsError::NotABody)?;
        transform.translation = position;
        transform.rotation = rotation;
        if let Some(mut p) = e.get_mut::<Position>() {
            p.0 = position;
        }
        if let Some(mut r) = e.get_mut::<Rotation>() {
            r.0 = rotation;
        }
        Ok(())
    })
}

/// Position-only teleport: keeps the current rotation and velocities.
pub fn body_set_position(entity: Entity, position: Vec3) -> Result<()> {
    app_mut(|app| {
        let mut e = app.world_mut().entity_mut(entity);
        let mut transform = e.get_mut::<Transform>().ok_or(PhysicsError::NotABody)?;
        transform.translation = position;
        if let Some(mut p) = e.get_mut::<Position>() {
            p.0 = position;
        }
        Ok(())
    })
}

pub fn body_set_linear_velocity(entity: Entity, velocity: Vec3) -> Result<()> {
    app_mut(|app| {
        let mut e = app.world_mut().entity_mut(entity);
        let mut v = e
            .get_mut::<LinearVelocity>()
            .ok_or(PhysicsError::NotABody)?;
        v.0 = velocity;
        Ok(())
    })
}

pub fn body_set_angular_velocity(entity: Entity, velocity: Vec3) -> Result<()> {
    app_mut(|app| {
        let mut e = app.world_mut().entity_mut(entity);
        let mut v = e
            .get_mut::<AngularVelocity>()
            .ok_or(PhysicsError::NotABody)?;
        v.0 = velocity;
        Ok(())
    })
}

pub fn body_apply_impulse(entity: Entity, impulse: Vec3) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(apply_impulse_system, (entity, impulse))
            .unwrap()
    })
}

fn apply_impulse_system(
    In((entity, impulse)): In<(Entity, Vec3)>,
    mut bodies: Query<Forces>,
) -> Result<()> {
    let mut forces = bodies.get_mut(entity).map_err(|_| PhysicsError::NotABody)?;
    forces.apply_linear_impulse(impulse);
    Ok(())
}

pub fn body_apply_angular_impulse(entity: Entity, impulse: Vec3) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(apply_angular_impulse_system, (entity, impulse))
            .unwrap()
    })
}

fn apply_angular_impulse_system(
    In((entity, impulse)): In<(Entity, Vec3)>,
    mut bodies: Query<Forces>,
) -> Result<()> {
    let mut forces = bodies.get_mut(entity).map_err(|_| PhysicsError::NotABody)?;
    forces.apply_angular_impulse(impulse);
    Ok(())
}

pub fn body_set_friction(entity: Entity, friction: f32) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .entity_mut(entity)
            .insert(Friction::new(friction));
        Ok(())
    })
}

pub fn body_set_restitution(entity: Entity, restitution: f32) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .entity_mut(entity)
            .insert(Restitution::new(restitution));
        Ok(())
    })
}

pub fn body_set_density(entity: Entity, density: f32) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .entity_mut(entity)
            .insert(ColliderDensity(density));
        Ok(())
    })
}

// ── Readback ────────────────────────────────────────────────────────────────

/// Position (3) + rotation quaternion xyzw (4) for one body.
pub fn body_get_transform(entity: Entity) -> Result<[f32; 7]> {
    app_mut(|app| {
        let e = app.world().entity(entity);
        let (pos, rot) = read_pose(&e).ok_or(PhysicsError::NotABody)?;
        Ok([pos.x, pos.y, pos.z, rot.x, rot.y, rot.z, rot.w])
    })
}

pub fn body_get_linear_velocity(entity: Entity) -> Result<[f32; 3]> {
    app_mut(|app| {
        let v = app
            .world()
            .entity(entity)
            .get::<LinearVelocity>()
            .ok_or(PhysicsError::NotABody)?
            .0;
        Ok([v.x, v.y, v.z])
    })
}

fn read_pose(e: &EntityRef) -> Option<(Vec3, Quat)> {
    // Prefer avian's own components (updated during the step); fall back to
    // Transform for bodies that haven't simulated yet.
    match (e.get::<Position>(), e.get::<Rotation>()) {
        (Some(p), Some(r)) => Some((p.0, r.0)),
        _ => e.get::<Transform>().map(|t| (t.translation, t.rotation)),
    }
}

/// Bulk pose readback: writes 7 floats (pos xyz, quat xyzw) per body into
/// `out`, which must hold `ids.len() * 7` floats. Bodies that no longer exist
/// write identity poses. Returns the number of bodies read.
pub fn read_transforms(ids: &[u64], out: &mut [f32]) -> Result<usize> {
    assert!(out.len() >= ids.len() * 7);
    app_mut(|app| {
        let world = app.world();
        let mut read = 0;
        for (i, &id) in ids.iter().enumerate() {
            let o = &mut out[i * 7..i * 7 + 7];
            let pose = world
                .get_entity(Entity::from_bits(id))
                .ok()
                .and_then(|e| read_pose(&e));
            match pose {
                Some((p, r)) => {
                    o.copy_from_slice(&[p.x, p.y, p.z, r.x, r.y, r.z, r.w]);
                    read += 1;
                }
                None => o.copy_from_slice(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            }
        }
        Ok(read)
    })
}
