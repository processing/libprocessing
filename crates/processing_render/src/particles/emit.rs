use std::sync::Mutex;

use bevy::prelude::*;

use processing_core::app_mut;
use processing_core::error;

use crate::geometry;
use crate::particles::grid::{Grid, GridParams, grid_bind, grid_build, grid_create};
use crate::particles::kernels::KernelRequires;
use crate::particles::{Particles, particles_ensure_attribute};
use crate::shader_value::ShaderValue;
use crate::{
    buffer_write_element, compute_create, compute_dispatch, compute_set,
    shader_load,
};

const WORKGROUP_SIZE: u32 = 64;

pub fn particles_emit_gpu(
    particles_entity: Entity,
    count: u32,
    compute_entity: Entity,
) -> error::Result<()> {
    if count == 0 {
        return Ok(());
    }

    let (capacity, head, buffers) = app_mut(|app| {
        let world = app.world();
        let field = world
            .get::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        if count > field.capacity {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "particles_emit_gpu count={} exceeds field capacity {}",
                count, field.capacity
            )));
        }
        let mut buffers: Vec<(String, Entity)> = Vec::with_capacity(field.buffers.len());
        for (&attr_entity, &buf_entity) in &field.buffers {
            let attr = world
                .get::<geometry::Attribute>(attr_entity)
                .ok_or(error::ProcessingError::InvalidEntity)?;
            buffers.push((attr.name.to_string(), buf_entity));
        }
        Ok((field.capacity, field.emit_head, buffers))
    })?;

    for (name, buf_entity) in buffers {
        match compute_set(compute_entity, name, ShaderValue::Buffer(buf_entity)) {
            Ok(()) => {}
            Err(error::ProcessingError::UnknownShaderProperty(_)) => {}
            Err(e) => return Err(e),
        }
    }

    for (name, value) in [
        ("emit_base", ShaderValue::UInt(head)),
        ("emit_count", ShaderValue::UInt(count)),
        ("emit_capacity", ShaderValue::UInt(capacity)),
    ] {
        match compute_set(compute_entity, name, value) {
            Ok(()) => {}
            Err(error::ProcessingError::UnknownShaderProperty(_)) => {}
            Err(e) => return Err(e),
        }
    }

    let workgroup_count = count.div_ceil(WORKGROUP_SIZE);
    compute_dispatch(compute_entity, workgroup_count, 1, 1)?;

    app_mut(|app| {
        let mut field = app
            .world_mut()
            .get_mut::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        field.emit_head = (field.emit_head + count) % field.capacity;
        Ok(())
    })
}

pub fn particles_emit(
    particles_entity: Entity,
    n: u32,
    attribute_data: Vec<(Entity, Vec<u8>)>,
) -> error::Result<()> {
    if n == 0 {
        return Ok(());
    }

    let (capacity, head, attr_specs) = app_mut(|app| {
        let world = app.world();
        let field = world
            .get::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        if n > field.capacity {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "particles_emit n={} exceeds field capacity {}",
                n, field.capacity
            )));
        }
        let mut specs: Vec<(Entity, u32, Entity)> = Vec::with_capacity(attribute_data.len());
        for (attr_entity, _) in &attribute_data {
            let attr = world
                .get::<geometry::Attribute>(*attr_entity)
                .ok_or(error::ProcessingError::InvalidEntity)?;
            let buf = field.buffer(*attr_entity).ok_or_else(|| {
                error::ProcessingError::InvalidArgument(format!(
                    "particles have no buffer for attribute {:?}",
                    attr_entity
                ))
            })?;
            specs.push((*attr_entity, attr.format.byte_size() as u32, buf));
        }
        Ok((field.capacity, field.emit_head, specs))
    })?;

    for ((_, bytes), &(_, byte_size, buf)) in attribute_data.iter().zip(attr_specs.iter()) {
        let expected = (n as usize) * (byte_size as usize);
        if bytes.len() != expected {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "expected {} bytes ({} particles * {} bytes), got {}",
                expected,
                n,
                byte_size,
                bytes.len()
            )));
        }
        let first_chunk_n = (capacity - head).min(n);
        let split = (first_chunk_n as usize) * (byte_size as usize);
        let first_offset = (head as u64) * (byte_size as u64);
        buffer_write_element(buf, first_offset, bytes[..split].to_vec())?;
        if first_chunk_n < n {
            buffer_write_element(buf, 0, bytes[split..].to_vec())?;
        }
    }

    app_mut(|app| {
        let mut field = app
            .world_mut()
            .get_mut::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        field.emit_head = (field.emit_head + n) % field.capacity;
        Ok(())
    })
}

pub fn particles_flock(
    particles_entity: Entity,
    flock_entity: Entity,
    grid: &Grid,
) -> error::Result<()> {
    let position = app_mut(|app| {
        let world = app.world();
        let field = world
            .get::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        for (&attr_entity, &buf_entity) in &field.buffers {
            let attr = world
                .get::<geometry::Attribute>(attr_entity)
                .ok_or(error::ProcessingError::InvalidEntity)?;
            if attr.name == "position" {
                return Ok(buf_entity);
            }
        }
        Err(error::ProcessingError::InvalidArgument(
            "particles_flock requires a `position` attribute".to_string(),
        ))
    })?;

    grid_build(grid, position)?;
    grid_bind(grid, flock_entity)?;
    particles_apply(particles_entity, flock_entity)
}

#[derive(Component, Clone, Copy)]
pub struct AutoFlockGrid(pub Grid);

#[derive(Component, Clone, Copy)]
pub struct NaiveFlockKernel(pub Entity);

const FLOCK_PARAMS: [&str; 8] = [
    "sep_distance",
    "neighbor_distance",
    "weight_separation",
    "weight_alignment",
    "weight_cohesion",
    "max_speed",
    "max_force",
    "min_speed",
];

pub fn particles_flock_auto(particles_entity: Entity, flock_entity: Entity) -> error::Result<()> {
    let (capacity, cell_size, cap, existing, naive) = app_mut(|app| {
        let world = app.world();
        let capacity = world
            .get::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?
            .capacity;
        let compute = world.get::<crate::compute::Compute>(flock_entity);
        let cell_size = compute
            .and_then(|c| c.shader.get::<f32>("neighbor_distance").copied())
            .unwrap_or(2.5)
            .max(1e-3);
        let cap = compute
            .and_then(|c| c.shader.get::<u32>("max_neighbors").copied())
            .unwrap_or(64);
        let existing = world.get::<AutoFlockGrid>(flock_entity).map(|g| g.0);
        let naive = world.get::<NaiveFlockKernel>(flock_entity).map(|n| n.0);
        Ok((capacity, cell_size, cap, existing, naive))
    })?;

    if cap == 0 {
        let naive = match naive {
            Some(e) => e,
            None => {
                let e = crate::particles::kernels::particles_kernel_flock_naive()?;
                app_mut(|app| {
                    app.world_mut()
                        .entity_mut(flock_entity)
                        .insert(NaiveFlockKernel(e));
                    Ok(())
                })?;
                e
            }
        };
        let values: Vec<(usize, f32)> = app_mut(|app| {
            let compute = app.world().get::<crate::compute::Compute>(flock_entity);
            Ok(FLOCK_PARAMS
                .iter()
                .enumerate()
                .filter_map(|(idx, name)| {
                    compute
                        .and_then(|c| c.shader.get::<f32>(name).copied())
                        .map(|v| (idx, v))
                })
                .collect())
        })?;
        for (idx, v) in values {
            compute_set(naive, FLOCK_PARAMS[idx], ShaderValue::Float(v))?;
        }
        return particles_apply(particles_entity, naive);
    }

    let reusable = existing.filter(|g| {
        g.capacity == capacity && (g.params.cell_size - cell_size).abs() < f32::EPSILON
    });
    let grid = match reusable {
        Some(g) => g,
        None => {
            if let Some(old) = existing {
                let _ = crate::buffer_destroy(old.offsets);
                let _ = crate::buffer_destroy(old.cursor);
                let _ = crate::buffer_destroy(old.sorted);
            }
            const HALF: f32 = 40.0;
            let dim = (((2.0 * HALF) / cell_size).ceil() as u32).clamp(1, 32);
            let half_extent = dim as f32 * cell_size / 2.0;
            let params = GridParams {
                min: [-half_extent; 3],
                cell_size,
                dims: [dim; 3],
            };
            let g = grid_create(params, capacity)?;
            app_mut(|app| {
                app.world_mut()
                    .entity_mut(flock_entity)
                    .insert(AutoFlockGrid(g));
                Ok(())
            })?;
            g
        }
    };
    particles_flock(particles_entity, flock_entity, &grid)
}

static NEIGHBOR_COMPUTE: Mutex<Option<Entity>> = Mutex::new(None);

fn neighbor_compute() -> error::Result<Entity> {
    let mut guard = NEIGHBOR_COMPUTE.lock().unwrap();
    if let Some(entity) = *guard {
        return Ok(entity);
    }
    let shader = shader_load("embedded://processing_render/particles/kernels/neighbor.wgsl")?;
    let entity = compute_create(shader)?;
    *guard = Some(entity);
    Ok(entity)
}

pub fn particles_gather(
    particles_entity: Entity,
    grid: &Grid,
    source: Entity,
    out: Entity,
    op: u32,
    radius: f32,
    falloff_mode: u32,
    components: u32,
) -> error::Result<()> {
    let (position, _capacity) = app_mut(|app| {
        let world = app.world();
        let field = world
            .get::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        for (&attr_entity, &buf_entity) in &field.buffers {
            let attr = world
                .get::<geometry::Attribute>(attr_entity)
                .ok_or(error::ProcessingError::InvalidEntity)?;
            if attr.name == "position" {
                return Ok((buf_entity, field.capacity));
            }
        }
        Err(error::ProcessingError::InvalidArgument(
            "apply(neighbor) requires a `position` attribute".to_string(),
        ))
    })?;

    if out == source || out == position {
        return Err(error::ProcessingError::InvalidArgument(
            "apply(neighbor): `out` must be a distinct buffer from the source and \
             position (in-place gather is a read/write hazard)"
                .to_string(),
        ));
    }

    grid_build(grid, position)?;
    let neighbor = neighbor_compute()?;
    grid_bind(grid, neighbor)?;
    compute_set(neighbor, "position", ShaderValue::Buffer(position))?;
    compute_set(neighbor, "source", ShaderValue::Buffer(source))?;
    compute_set(neighbor, "out", ShaderValue::Buffer(out))?;
    compute_set(neighbor, "radius", ShaderValue::Float(radius))?;
    compute_set(neighbor, "op", ShaderValue::UInt(op))?;
    compute_set(neighbor, "falloff_mode", ShaderValue::UInt(falloff_mode))?;
    compute_set(neighbor, "components", ShaderValue::UInt(components))?;

    let capacity = _capacity;
    compute_dispatch(neighbor, capacity.div_ceil(WORKGROUP_SIZE), 1, 1)
}

pub fn particles_apply(particles_entity: Entity, compute_entity: Entity) -> error::Result<()> {
    let required: Vec<Entity> = app_mut(|app| {
        Ok(app
            .world()
            .get::<KernelRequires>(compute_entity)
            .map(|r| r.0.clone())
            .unwrap_or_default())
    })?;
    for attr_entity in required {
        particles_ensure_attribute(particles_entity, attr_entity)?;
    }

    let (capacity, buffers) = app_mut(|app| {
        let world = app.world();
        let field = world
            .get::<Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        let mut buffers: Vec<(String, Entity)> = Vec::with_capacity(field.buffers.len());
        for (&attr_entity, &buf_entity) in &field.buffers {
            let attr = world
                .get::<geometry::Attribute>(attr_entity)
                .ok_or(error::ProcessingError::InvalidEntity)?;
            buffers.push((attr.name.to_string(), buf_entity));
        }
        Ok((field.capacity, buffers))
    })?;

    for (name, buf_entity) in buffers {
        match compute_set(compute_entity, name, ShaderValue::Buffer(buf_entity)) {
            Ok(()) => {}
            Err(error::ProcessingError::UnknownShaderProperty(_)) => {}
            Err(e) => return Err(e),
        }
    }

    let workgroup_count = capacity.div_ceil(WORKGROUP_SIZE);
    compute_dispatch(compute_entity, workgroup_count, 1, 1)
}
