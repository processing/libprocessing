use std::sync::Mutex;

use bevy::prelude::Entity;

use processing_core::error::Result;

use crate::shader_value::ShaderValue;
use crate::{
    buffer_create, buffer_destroy, buffer_read_element, buffer_size, compute_create,
    compute_dispatch, compute_set, shader_load,
};

const BLOCK: u32 = 256;
const SHADER: &str = "embedded://processing_render/particles/kernels/reduce.wgsl";

pub const REDUCE_OP_SUM: u32 = 0;
pub const REDUCE_OP_MIN: u32 = 1;
pub const REDUCE_OP_MAX: u32 = 2;

static COMPUTE: Mutex<Option<Entity>> = Mutex::new(None);
static SCRATCH: Mutex<Vec<(Entity, u64)>> = Mutex::new(Vec::new());

fn reduce_compute() -> Result<Entity> {
    let mut guard = COMPUTE.lock().unwrap();
    if let Some(e) = *guard {
        return Ok(e);
    }
    let compute = compute_create(shader_load(SHADER)?)?;
    *guard = Some(compute);
    Ok(compute)
}

fn scratch(level: usize, bytes: u64) -> Result<Entity> {
    let mut scratch = SCRATCH.lock().unwrap();
    while scratch.len() <= level {
        scratch.push((Entity::PLACEHOLDER, 0));
    }
    let (entity, size) = scratch[level];
    if entity != Entity::PLACEHOLDER && size >= bytes {
        return Ok(entity);
    }
    if entity != Entity::PLACEHOLDER {
        let _ = buffer_destroy(entity);
    }
    let new_entity = buffer_create(bytes)?;
    scratch[level] = (new_entity, bytes);
    Ok(new_entity)
}

pub fn reduce(values: Entity, op: u32) -> Result<f32> {
    let n = (buffer_size(values)? / 4) as u32;
    if n == 0 {
        return Ok(match op {
            REDUCE_OP_MIN => f32::INFINITY,
            REDUCE_OP_MAX => f32::NEG_INFINITY,
            _ => 0.0,
        });
    }

    let compute = reduce_compute()?;
    let mut cur = values;
    let mut cur_n = n;
    let mut level = 0usize;

    loop {
        let num_wg = cur_n.div_ceil(BLOCK);
        let out = scratch(level, (num_wg as u64) * 4)?;

        compute_set(compute, "input", ShaderValue::Buffer(cur))?;
        compute_set(compute, "output", ShaderValue::Buffer(out))?;
        compute_set(compute, "mode", ShaderValue::UInt(op))?;
        compute_set(compute, "count", ShaderValue::UInt(cur_n))?;
        compute_dispatch(compute, num_wg, 1, 1)?;

        if num_wg == 1 {
            let bytes = buffer_read_element(out, 0, 4)?;
            return Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
        }
        cur = out;
        cur_n = num_wg;
        level += 1;
    }
}
