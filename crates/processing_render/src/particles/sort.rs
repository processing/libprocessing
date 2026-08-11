use std::sync::Mutex;

use bevy::prelude::Entity;

use processing_core::error::{ProcessingError, Result};

use crate::shader_value::ShaderValue;
use crate::{buffer_size, compute_create, compute_dispatch, compute_set, shader_load};

const WG: u32 = 64;
const SHADER: &str = "embedded://processing_render/particles/kernels/bitonic.wgsl";

static BITONIC: Mutex<Option<Entity>> = Mutex::new(None);

fn bitonic_compute() -> Result<Entity> {
    let mut guard = BITONIC.lock().unwrap();
    if let Some(e) = *guard {
        return Ok(e);
    }
    let compute = compute_create(shader_load(SHADER)?)?;
    *guard = Some(compute);
    Ok(compute)
}

pub fn bitonic_sort_by_key(keys: Entity, payload: Entity) -> Result<()> {
    let n = (buffer_size(keys)? / 4) as u32;
    if n <= 1 {
        return Ok(());
    }
    if !n.is_power_of_two() {
        return Err(ProcessingError::InvalidArgument(format!(
            "bitonic_sort_by_key: length {n} must be a power of two"
        )));
    }
    if buffer_size(payload)? / 4 != n as u64 {
        return Err(ProcessingError::InvalidArgument(
            "bitonic_sort_by_key: keys and payload must have the same length".to_string(),
        ));
    }

    let compute = bitonic_compute()?;
    let workgroups = n.div_ceil(WG);

    let mut k = 2u32;
    while k <= n {
        let mut j = k / 2;
        while j >= 1 {
            compute_set(compute, "keys", ShaderValue::Buffer(keys))?;
            compute_set(compute, "payload", ShaderValue::Buffer(payload))?;
            compute_set(compute, "k", ShaderValue::UInt(k))?;
            compute_set(compute, "j", ShaderValue::UInt(j))?;
            compute_dispatch(compute, workgroups, 1, 1)?;
            j /= 2;
        }
        k *= 2;
    }
    Ok(())
}
