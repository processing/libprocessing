use std::sync::Mutex;

use bevy::prelude::Entity;

use processing_core::app_mut;
use processing_core::error::{ProcessingError, Result};

use crate::particles::grid::grid_build;
use crate::particles::{NeighborLists, Particles};
use crate::shader_value::ShaderValue;
use crate::{
    buffer_create, buffer_destroy, compute_create, compute_dispatch_no_update, compute_set,
    geometry_attribute_position, shader_load,
};

const BUILD_SHADER: &str = "embedded://processing_render/particles/kernels/neighbors_build.wgsl";
const REDUCE_SHADER: &str = "embedded://processing_render/particles/kernels/neighbors_reduce.wgsl";
const WG: u32 = 64;

static COMPUTES: Mutex<Option<(Entity, Entity)>> = Mutex::new(None);

fn computes() -> Result<(Entity, Entity)> {
    let mut guard = COMPUTES.lock().unwrap();
    if let Some(v) = *guard {
        return Ok(v);
    }
    let build = compute_create(shader_load(BUILD_SHADER)?)?;
    let reduce = compute_create(shader_load(REDUCE_SHADER)?)?;
    *guard = Some((build, reduce));
    Ok((build, reduce))
}

fn position_buffer(particles: Entity) -> Result<Entity> {
    crate::particles::particles_buffer(particles, geometry_attribute_position())?.ok_or_else(|| {
        ProcessingError::InvalidArgument("neighbors need a `position` attribute".to_string())
    })
}

/// The `neighbors` and `neighbor_count` attributes, if `particles_find_neighbors` has run.
pub fn particles_neighbor_lists(particles: Entity) -> Result<Option<NeighborLists>> {
    app_mut(|app| {
        Ok(app
            .world()
            .get::<Particles>(particles)
            .ok_or(ProcessingError::ParticlesNotFound)?
            .neighbor_lists)
    })
}

/// Rebuilds `grid` from the particles' positions, then writes each particle's `neighbors`
/// (up to `max` within `radius`) and `neighbor_count`. Past `max`, a spread sample is kept,
/// not the nearest.
pub fn particles_find_neighbors(
    particles: Entity,
    grid: Entity,
    radius: f32,
    max: u32,
) -> Result<()> {
    if max == 0 {
        return Err(ProcessingError::InvalidArgument(
            "find_neighbors: max must be at least 1".to_string(),
        ));
    }
    let capacity = crate::particles::particles_capacity(particles)?;
    let lists = match particles_neighbor_lists(particles)? {
        Some(lists) if lists.max == max => lists,
        previous => {
            if let Some(previous) = previous {
                buffer_destroy(previous.neighbors)?;
                buffer_destroy(previous.count)?;
            }
            let slots = capacity.max(1) as u64;
            let lists = NeighborLists {
                neighbors: buffer_create(slots * max as u64 * 4)?,
                count: buffer_create(slots * 4)?,
                max,
            };
            app_mut(|app| {
                app.world_mut()
                    .get_mut::<Particles>(particles)
                    .ok_or(ProcessingError::ParticlesNotFound)?
                    .neighbor_lists = Some(lists);
                Ok(())
            })?;
            lists
        }
    };

    let position = position_buffer(particles)?;
    grid_build(grid, position)?;
    let (build, _) = computes()?;
    compute_set(build, "position", ShaderValue::Buffer(position))?;
    compute_set(build, "grid", ShaderValue::Grid(grid))?;
    compute_set(build, "neighbors", ShaderValue::Buffer(lists.neighbors))?;
    compute_set(build, "neighbor_count", ShaderValue::Buffer(lists.count))?;
    compute_set(build, "radius", ShaderValue::Float(radius))?;
    compute_set(build, "max_neighbors", ShaderValue::UInt(max))?;
    compute_dispatch_no_update(build, capacity.div_ceil(WG), 1, 1)
}

/// Like `particles_gather`, but over each particle's `neighbors`, so a particle never counts
/// itself. With `relative`, reduces `source[j] - source[i]`.
#[allow(clippy::too_many_arguments)]
pub fn particles_neighbor_reduce(
    particles: Entity,
    source: Entity,
    out: Entity,
    op: u32,
    radius: f32,
    falloff_mode: u32,
    components: u32,
    relative: bool,
) -> Result<()> {
    let lists = particles_neighbor_lists(particles)?.ok_or_else(|| {
        ProcessingError::InvalidArgument("run find_neighbors before reading neighbors".to_string())
    })?;
    let position = position_buffer(particles)?;
    if out == source || out == position {
        return Err(ProcessingError::InvalidArgument(
            "neighbor: `out` must differ from the source and position".to_string(),
        ));
    }
    let capacity = crate::particles::particles_capacity(particles)?;
    let (_, reduce) = computes()?;
    compute_set(reduce, "position", ShaderValue::Buffer(position))?;
    compute_set(reduce, "source", ShaderValue::Buffer(source))?;
    compute_set(reduce, "out", ShaderValue::Buffer(out))?;
    compute_set(reduce, "neighbors", ShaderValue::Buffer(lists.neighbors))?;
    compute_set(reduce, "neighbor_count", ShaderValue::Buffer(lists.count))?;
    compute_set(reduce, "max_distance", ShaderValue::Float(radius))?;
    compute_set(reduce, "op", ShaderValue::UInt(op))?;
    compute_set(reduce, "falloff_mode", ShaderValue::UInt(falloff_mode))?;
    compute_set(reduce, "components", ShaderValue::UInt(components))?;
    compute_set(reduce, "relative", ShaderValue::UInt(relative as u32))?;
    compute_dispatch_no_update(reduce, capacity.div_ceil(WG), 1, 1)
}
