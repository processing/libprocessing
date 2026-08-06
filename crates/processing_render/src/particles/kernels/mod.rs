//! built-in compute kernels for [`Particles`](super::Particles), embedded as
//! assets and dispatched via `particles_apply`.

use bevy::asset::embedded_asset;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use processing_core::app_mut;
use processing_core::error::{self, ProcessingError};

use crate::geometry::{BuiltinAttributes, Geometry};
use crate::shader_value::ShaderValue;
use crate::{compute_create, compute_set, shader_load};

#[derive(Component, Default, Clone)]
pub struct KernelRequires(pub Vec<Entity>);

fn set_requires(compute: Entity, names: &[&str]) -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        let attrs: Vec<Entity> = {
            let builtins = world.resource::<BuiltinAttributes>();
            names.iter().filter_map(|n| builtins.by_name(n)).collect()
        };
        world.entity_mut(compute).insert(KernelRequires(attrs));
        Ok(())
    })
}

pub struct ParticlesKernelsPlugin;

impl Plugin for ParticlesKernelsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "noise.wgsl");
        embedded_asset!(app, "transform.wgsl");
        embedded_asset!(app, "attract.wgsl");
        embedded_asset!(app, "drag.wgsl");
        embedded_asset!(app, "force.wgsl");
        embedded_asset!(app, "integrate.wgsl");
        embedded_asset!(app, "age.wgsl");
        embedded_asset!(app, "vortex.wgsl");
        embedded_asset!(app, "bounds_sphere.wgsl");
        embedded_asset!(app, "bounds_box.wgsl");
        embedded_asset!(app, "impulse.wgsl");
        embedded_asset!(app, "flock.wgsl");
        embedded_asset!(app, "orient.wgsl");
        embedded_asset!(app, "field.wgsl");
        embedded_asset!(app, "attr_linear.wgsl");
        embedded_asset!(app, "attr_combine.wgsl");
        embedded_asset!(app, "attr_mix.wgsl");
        embedded_asset!(app, "attr_lookup1d.wgsl");
        embedded_asset!(app, "attr_lookup2d.wgsl");
        embedded_asset!(app, "scatter_surface.wgsl");
        embedded_asset!(app, "scatter_volume.wgsl");
    }
}

pub const FALLOFF_CONST: u32 = 0;
pub const FALLOFF_LINEAR: u32 = 1;
pub const FALLOFF_SMOOTHSTEP: u32 = 2;
pub const FALLOFF_QUADRATIC: u32 = 3;
pub const FALLOFF_CUBIC: u32 = 4;
pub const FALLOFF_INVERSE: u32 = 5;

pub const BOUNDS_CLAMP: u32 = 0;
pub const BOUNDS_REFLECT: u32 = 1;
pub const BOUNDS_WRAP: u32 = 2;
pub const BOUNDS_SOFT: u32 = 3;

pub const COMBINE_ADD: u32 = 0;
pub const COMBINE_SUB: u32 = 1;
pub const COMBINE_MUL: u32 = 2;
pub const COMBINE_DIV: u32 = 3;
pub const COMBINE_MIN: u32 = 4;
pub const COMBINE_MAX: u32 = 5;
pub const COMBINE_POW: u32 = 6;

pub fn particles_kernel_noise() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/noise.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position"])?;
    compute_set(entity, "divergence_free", ShaderValue::UInt(0))?;
    Ok(entity)
}

pub fn particles_kernel_transform() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/transform.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position"])?;
    compute_set(entity, "translate", ShaderValue::Float3([0.0; 3]))?;
    compute_set(
        entity,
        "rotation_axis",
        ShaderValue::Float3([0.0, 1.0, 0.0]),
    )?;
    compute_set(entity, "rotation_angle", ShaderValue::Float(0.0))?;
    compute_set(entity, "scale", ShaderValue::Float3([1.0, 1.0, 1.0]))?;
    Ok(entity)
}

pub fn particles_kernel_attract() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/attract.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position", "velocity"])?;
    compute_set(entity, "center", ShaderValue::Float3([0.0; 3]))?;
    compute_set(entity, "strength", ShaderValue::Float(0.0))?;
    compute_set(entity, "radius", ShaderValue::Float(1.0))?;
    compute_set(entity, "falloff_mode", ShaderValue::UInt(FALLOFF_LINEAR))?;
    Ok(entity)
}

pub fn particles_kernel_drag() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/drag.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["velocity"])?;
    compute_set(entity, "damping", ShaderValue::Float(0.0))?;
    compute_set(entity, "max_speed", ShaderValue::Float(0.0))?;
    Ok(entity)
}

pub fn particles_kernel_force() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/force.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["velocity"])?;
    compute_set(entity, "direction", ShaderValue::Float3([0.0, -1.0, 0.0]))?;
    compute_set(entity, "strength", ShaderValue::Float(0.0))?;
    Ok(entity)
}

pub fn particles_kernel_integrate() -> error::Result<Entity> {
    let shader =
        shader_load("embedded://processing_render/particles/kernels/integrate.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position", "velocity"])?;
    compute_set(entity, "dt", ShaderValue::Float(1.0))?;
    Ok(entity)
}

pub fn particles_kernel_age() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/age.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["age", "life"])?;
    compute_set(entity, "dt", ShaderValue::Float(1.0))?;
    Ok(entity)
}

pub fn particles_kernel_vortex() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/vortex.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position", "velocity"])?;
    compute_set(entity, "center", ShaderValue::Float3([0.0; 3]))?;
    compute_set(entity, "axis", ShaderValue::Float3([0.0, 1.0, 0.0]))?;
    compute_set(entity, "strength", ShaderValue::Float(0.0))?;
    compute_set(entity, "radius", ShaderValue::Float(1.0))?;
    compute_set(entity, "falloff_mode", ShaderValue::UInt(FALLOFF_LINEAR))?;
    Ok(entity)
}

pub fn particles_kernel_bounds_sphere() -> error::Result<Entity> {
    let shader =
        shader_load("embedded://processing_render/particles/kernels/bounds_sphere.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position", "velocity"])?;
    compute_set(entity, "center", ShaderValue::Float3([0.0; 3]))?;
    compute_set(entity, "radius", ShaderValue::Float(1.0))?;
    compute_set(entity, "mode", ShaderValue::UInt(BOUNDS_SOFT))?;
    compute_set(entity, "soft_strength", ShaderValue::Float(0.01))?;
    compute_set(entity, "max_speed", ShaderValue::Float(0.0))?;
    Ok(entity)
}

pub fn particles_kernel_bounds_box() -> error::Result<Entity> {
    let shader =
        shader_load("embedded://processing_render/particles/kernels/bounds_box.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position", "velocity"])?;
    compute_set(entity, "aabb_min", ShaderValue::Float3([-1.0, -1.0, -1.0]))?;
    compute_set(entity, "aabb_max", ShaderValue::Float3([1.0, 1.0, 1.0]))?;
    compute_set(entity, "mode", ShaderValue::UInt(BOUNDS_SOFT))?;
    compute_set(entity, "soft_strength", ShaderValue::Float(0.01))?;
    compute_set(entity, "max_speed", ShaderValue::Float(0.0))?;
    Ok(entity)
}

pub fn particles_kernel_bounds_geometry(geometry_entity: Entity) -> error::Result<Entity> {
    let (aabb_min, aabb_max) = app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(extract_geometry_aabb, geometry_entity)
            .unwrap()
    })?;
    let entity = particles_kernel_bounds_box()?;
    compute_set(entity, "aabb_min", ShaderValue::Float3(aabb_min))?;
    compute_set(entity, "aabb_max", ShaderValue::Float3(aabb_max))?;
    Ok(entity)
}

fn extract_geometry_aabb(
    In(geom_entity): In<Entity>,
    geometries: Query<&Geometry>,
    meshes: Res<Assets<Mesh>>,
) -> error::Result<([f32; 3], [f32; 3])> {
    let geom = geometries
        .get(geom_entity)
        .map_err(|_| ProcessingError::GeometryNotFound)?;
    let mesh = meshes
        .get(&geom.handle)
        .ok_or(ProcessingError::GeometryNotFound)?;
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(p)) => p,
        _ => {
            return Err(ProcessingError::InvalidArgument(
                "bounds geometry has no Float32x3 position attribute".to_string(),
            ));
        }
    };
    if positions.is_empty() {
        return Err(ProcessingError::InvalidArgument(
            "bounds geometry has no vertices".to_string(),
        ));
    }
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for p in positions {
        let v = Vec3::from_array(*p);
        min = min.min(v);
        max = max.max(v);
    }
    Ok((min.to_array(), max.to_array()))
}

pub fn particles_kernel_impulse() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/impulse.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position", "velocity"])?;
    compute_set(entity, "center", ShaderValue::Float3([0.0; 3]))?;
    compute_set(entity, "radius", ShaderValue::Float(1.0))?;
    compute_set(entity, "position_kick", ShaderValue::Float(0.0))?;
    compute_set(entity, "velocity_kick", ShaderValue::Float(0.0))?;
    compute_set(entity, "falloff_mode", ShaderValue::UInt(FALLOFF_SMOOTHSTEP))?;
    Ok(entity)
}

pub fn particles_kernel_flock() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/flock.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position", "velocity"])?;
    compute_set(entity, "sep_distance", ShaderValue::Float(1.2))?;
    compute_set(entity, "neighbor_distance", ShaderValue::Float(2.5))?;
    compute_set(entity, "weight_separation", ShaderValue::Float(1.5))?;
    compute_set(entity, "weight_alignment", ShaderValue::Float(1.0))?;
    compute_set(entity, "weight_cohesion", ShaderValue::Float(1.0))?;
    compute_set(entity, "max_speed", ShaderValue::Float(0.1))?;
    compute_set(entity, "max_force", ShaderValue::Float(0.003))?;
    compute_set(entity, "min_speed", ShaderValue::Float(0.02))?;
    Ok(entity)
}

pub fn particles_kernel_orient() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/orient.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["velocity", "rotation"])?;
    compute_set(entity, "forward", ShaderValue::Float3([0.0, 0.0, 1.0]))?;
    compute_set(entity, "up", ShaderValue::Float3([0.0, 1.0, 0.0]))?;
    Ok(entity)
}

pub fn particles_kernel_field() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/field.wgsl")?;
    let entity = compute_create(shader)?;
    set_requires(entity, &["position"])?;
    compute_set(entity, "center", ShaderValue::Float3([0.0; 3]))?;
    compute_set(entity, "radius", ShaderValue::Float(1.0))?;
    compute_set(entity, "falloff_mode", ShaderValue::UInt(FALLOFF_SMOOTHSTEP))?;
    Ok(entity)
}

pub fn particles_kernel_attr_linear() -> error::Result<Entity> {
    let shader =
        shader_load("embedded://processing_render/particles/kernels/attr_linear.wgsl")?;
    let entity = compute_create(shader)?;
    compute_set(entity, "scale", ShaderValue::Float(1.0))?;
    compute_set(entity, "offset", ShaderValue::Float(0.0))?;
    Ok(entity)
}

pub fn particles_kernel_attr_combine() -> error::Result<Entity> {
    let shader =
        shader_load("embedded://processing_render/particles/kernels/attr_combine.wgsl")?;
    let entity = compute_create(shader)?;
    compute_set(entity, "op", ShaderValue::UInt(COMBINE_ADD))?;
    compute_set(entity, "b_scale", ShaderValue::Float(1.0))?;
    compute_set(entity, "b_offset", ShaderValue::Float(0.0))?;
    Ok(entity)
}

pub fn particles_kernel_attr_mix() -> error::Result<Entity> {
    let shader = shader_load("embedded://processing_render/particles/kernels/attr_mix.wgsl")?;
    let entity = compute_create(shader)?;
    compute_set(entity, "t_scale", ShaderValue::Float(1.0))?;
    compute_set(entity, "t_offset", ShaderValue::Float(0.0))?;
    compute_set(entity, "t_clamp", ShaderValue::UInt(1))?;
    Ok(entity)
}

pub fn particles_kernel_attr_lookup1d() -> error::Result<Entity> {
    let shader =
        shader_load("embedded://processing_render/particles/kernels/attr_lookup1d.wgsl")?;
    let entity = compute_create(shader)?;
    compute_set(entity, "scale", ShaderValue::Float(1.0))?;
    compute_set(entity, "offset", ShaderValue::Float(0.0))?;
    Ok(entity)
}

pub fn particles_kernel_attr_lookup2d() -> error::Result<Entity> {
    let shader =
        shader_load("embedded://processing_render/particles/kernels/attr_lookup2d.wgsl")?;
    let entity = compute_create(shader)?;
    compute_set(entity, "u_scale", ShaderValue::Float(1.0))?;
    compute_set(entity, "u_offset", ShaderValue::Float(0.0))?;
    compute_set(entity, "v_scale", ShaderValue::Float(1.0))?;
    compute_set(entity, "v_offset", ShaderValue::Float(0.0))?;
    compute_set(entity, "color_scale", ShaderValue::Float(1.0))?;
    Ok(entity)
}
