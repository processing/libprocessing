use std::sync::Mutex;

use bevy::prelude::{Component, Entity};

use processing_core::app_mut;
use processing_core::error::{ProcessingError, Result};

use crate::particles::scan::prefix_sum_u32;
use crate::shader_value::ShaderValue;
use crate::{
    buffer_create, buffer_destroy, compute_create, compute_dispatch_no_update, compute_set,
    shader_load,
};

const CLEAR_SHADER: &str = "embedded://processing_render/particles/kernels/grid_clear.wgsl";
const COUNT_SHADER: &str = "embedded://processing_render/particles/kernels/grid_count.wgsl";
const COPY_SHADER: &str = "embedded://processing_render/particles/kernels/grid_copy.wgsl";
const SCATTER_SHADER: &str = "embedded://processing_render/particles/kernels/grid_scatter.wgsl";

static GRID_COMPUTES: Mutex<Option<(Entity, Entity, Entity, Entity)>> = Mutex::new(None);

fn grid_computes() -> Result<(Entity, Entity, Entity, Entity)> {
    let mut guard = GRID_COMPUTES.lock().unwrap();
    if let Some(v) = *guard {
        return Ok(v);
    }
    let clear = compute_create(shader_load(CLEAR_SHADER)?)?;
    let count = compute_create(shader_load(COUNT_SHADER)?)?;
    let copy = compute_create(shader_load(COPY_SHADER)?)?;
    let scatter = compute_create(shader_load(SCATTER_SHADER)?)?;
    *guard = Some((clear, count, copy, scatter));
    Ok((clear, count, copy, scatter))
}

#[derive(Clone, Copy, Debug)]
pub struct GridParams {
    pub min: [f32; 3],
    pub cell_size: f32,
    pub dims: [u32; 3],
}

impl GridParams {
    pub fn num_cells(&self) -> u32 {
        self.dims[0] * self.dims[1] * self.dims[2]
    }
}

#[derive(Component, Clone, Copy)]
pub struct Grid {
    pub offsets: Entity,
    pub cursor: Entity,
    pub sorted: Entity,
    pub params: GridParams,
    pub capacity: u32,
}

const CLEAR_WG: u32 = 256;
const PARTICLE_WG: u32 = 64;
const COPY_WG: u32 = 256;

pub fn grid_create(params: GridParams, capacity: u32) -> Result<Entity> {
    let num_cells = params.num_cells();
    let grid = Grid {
        offsets: buffer_create(((num_cells + 1) as u64) * 4)?,
        cursor: buffer_create((num_cells as u64) * 4)?,
        sorted: buffer_create((capacity.max(1) as u64) * 4)?,
        params,
        capacity,
    };
    app_mut(|app| Ok(app.world_mut().spawn(grid).id()))
}

pub fn grid_get(entity: Entity) -> Result<Grid> {
    app_mut(|app| {
        app.world()
            .get::<Grid>(entity)
            .copied()
            .ok_or(ProcessingError::GridNotFound)
    })
}

pub fn grid_destroy(entity: Entity) -> Result<()> {
    let grid = grid_get(entity)?;
    buffer_destroy(grid.offsets)?;
    buffer_destroy(grid.cursor)?;
    buffer_destroy(grid.sorted)?;
    app_mut(|app| {
        app.world_mut().despawn(entity);
        Ok(())
    })
}

fn set_domain(compute: Entity, params: &GridParams) -> Result<()> {
    compute_set(compute, "grid_min", ShaderValue::Float3(params.min))?;
    compute_set(compute, "cell_size", ShaderValue::Float(params.cell_size))?;
    compute_set(compute, "dims_x", ShaderValue::UInt(params.dims[0]))?;
    compute_set(compute, "dims_y", ShaderValue::UInt(params.dims[1]))?;
    compute_set(compute, "dims_z", ShaderValue::UInt(params.dims[2]))?;
    Ok(())
}

pub fn grid_build(entity: Entity, position: Entity) -> Result<()> {
    let grid = grid_get(entity)?;
    let (clear, count, copy, scatter) = grid_computes()?;
    let num_cells = grid.params.num_cells();

    compute_set(clear, "counts", ShaderValue::Buffer(grid.offsets))?;
    compute_dispatch_no_update(clear, (num_cells + 1).div_ceil(CLEAR_WG), 1, 1)?;

    compute_set(count, "position", ShaderValue::Buffer(position))?;
    compute_set(count, "counts", ShaderValue::Buffer(grid.offsets))?;
    set_domain(count, &grid.params)?;
    compute_dispatch_no_update(count, grid.capacity.div_ceil(PARTICLE_WG), 1, 1)?;

    prefix_sum_u32(grid.offsets)?;

    compute_set(copy, "starts", ShaderValue::Buffer(grid.offsets))?;
    compute_set(copy, "cursor", ShaderValue::Buffer(grid.cursor))?;
    compute_dispatch_no_update(copy, num_cells.div_ceil(COPY_WG), 1, 1)?;

    compute_set(scatter, "position", ShaderValue::Buffer(position))?;
    compute_set(scatter, "cursor", ShaderValue::Buffer(grid.cursor))?;
    compute_set(scatter, "sorted", ShaderValue::Buffer(grid.sorted))?;
    set_domain(scatter, &grid.params)?;
    compute_dispatch_no_update(scatter, grid.capacity.div_ceil(PARTICLE_WG), 1, 1)?;

    Ok(())
}
