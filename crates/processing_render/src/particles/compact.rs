use std::sync::Mutex;

use bevy::prelude::Entity;

use processing_core::error::Result;

use crate::particles::scan::prefix_sum_u32;
use crate::shader_value::ShaderValue;
use crate::{
    buffer_create, buffer_destroy, buffer_read_element, buffer_size, compute_create,
    compute_dispatch, compute_set, shader_load,
};

const WG: u32 = 64;
const FLAG_SHADER: &str = "embedded://processing_render/particles/kernels/compact_flag.wgsl";
const SCATTER_SHADER: &str = "embedded://processing_render/particles/kernels/compact_scatter.wgsl";

static COMPUTES: Mutex<Option<(Entity, Entity)>> = Mutex::new(None);
static SCANNED: Mutex<Option<(Entity, u64)>> = Mutex::new(None);

fn computes() -> Result<(Entity, Entity)> {
    let mut guard = COMPUTES.lock().unwrap();
    if let Some(v) = *guard {
        return Ok(v);
    }
    let flag = compute_create(shader_load(FLAG_SHADER)?)?;
    let scatter = compute_create(shader_load(SCATTER_SHADER)?)?;
    *guard = Some((flag, scatter));
    Ok((flag, scatter))
}

fn scanned_buffer(bytes: u64) -> Result<Entity> {
    let mut guard = SCANNED.lock().unwrap();
    if let Some((entity, size)) = *guard {
        if size == bytes {
            return Ok(entity);
        }
        let _ = buffer_destroy(entity);
    }
    let entity = buffer_create(bytes)?;
    *guard = Some((entity, bytes));
    Ok(entity)
}

pub fn compact(flags: Entity, indices_out: Entity) -> Result<u32> {
    let n = (buffer_size(flags)? / 4) as u32;
    if n == 0 {
        return Ok(0);
    }
    let (flag_c, scatter_c) = computes()?;
    let scanned = scanned_buffer(((n + 1) as u64) * 4)?;

    compute_set(flag_c, "flags", ShaderValue::Buffer(flags))?;
    compute_set(flag_c, "scanned", ShaderValue::Buffer(scanned))?;
    compute_dispatch(flag_c, (n + 1).div_ceil(WG), 1, 1)?;

    prefix_sum_u32(scanned)?;

    compute_set(scatter_c, "flags", ShaderValue::Buffer(flags))?;
    compute_set(scatter_c, "scanned", ShaderValue::Buffer(scanned))?;
    compute_set(scatter_c, "indices", ShaderValue::Buffer(indices_out))?;
    compute_dispatch(scatter_c, n.div_ceil(WG), 1, 1)?;

    let bytes = buffer_read_element(scanned, (n as u64) * 4, 4)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
