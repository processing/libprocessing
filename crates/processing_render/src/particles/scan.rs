use std::sync::Mutex;

use bevy::prelude::Entity;

use processing_core::error::Result;

use crate::shader_value::ShaderValue;
use crate::{
    buffer_create, buffer_destroy, buffer_size, compute_create, compute_dispatch_quiet, compute_set,
    shader_load,
};

const BLOCK: u64 = 256;

const BLOCK_SHADER: &str = "embedded://processing_render/particles/kernels/scan_block.wgsl";
const ADD_SHADER: &str = "embedded://processing_render/particles/kernels/scan_add.wgsl";

static SCAN_COMPUTES: Mutex<Option<(Entity, Entity)>> = Mutex::new(None);

static SCRATCH: Mutex<Vec<(Entity, u64)>> = Mutex::new(Vec::new());

fn scan_computes() -> Result<(Entity, Entity)> {
    let mut guard = SCAN_COMPUTES.lock().unwrap();
    if let Some(v) = *guard {
        return Ok(v);
    }
    let block = compute_create(shader_load(BLOCK_SHADER)?)?;
    let add = compute_create(shader_load(ADD_SHADER)?)?;
    *guard = Some((block, add));
    Ok((block, add))
}

fn scratch_buffer(level: usize, bytes: u64) -> Result<Entity> {
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

pub fn prefix_sum_u32(buffer: Entity) -> Result<()> {
    let total_bytes = buffer_size(buffer)?;
    if total_bytes == 0 {
        return Ok(());
    }

    let (block, add) = scan_computes()?;

    let n = (total_bytes / 4).max(1);

    let mut level_bufs: Vec<Entity> = vec![buffer];
    let mut level_ns: Vec<u64> = vec![n];

    let mut lvl = 0usize;
    loop {
        let num_blocks = level_ns[lvl].div_ceil(BLOCK).max(1);
        let sums = scratch_buffer(lvl, num_blocks * 4)?;

        compute_set(block, "data", ShaderValue::Buffer(level_bufs[lvl]))?;
        compute_set(block, "block_sums", ShaderValue::Buffer(sums))?;
        compute_dispatch_quiet(block, num_blocks as u32, 1, 1)?;

        level_bufs.push(sums);
        level_ns.push(num_blocks);

        if num_blocks == 1 {
            break;
        }
        lvl += 1;
    }

    for k in (0..level_bufs.len() - 2).rev() {
        let num_blocks = level_ns[k].div_ceil(BLOCK).max(1);
        compute_set(add, "data", ShaderValue::Buffer(level_bufs[k]))?;
        compute_set(add, "block_sums", ShaderValue::Buffer(level_bufs[k + 1]))?;
        compute_dispatch_quiet(add, num_blocks as u32, 1, 1)?;
    }

    Ok(())
}
