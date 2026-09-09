//! 3D texture FFI: a volume texture writable from compute kernels
//! (`texture_storage_3d<FMT, write>`) and sampled with trilinear filtering in
//! custom materials/filters (`texture_3d<f32>`). Bound by name like any other
//! texture via `processing_shader_set_texture` / material texture params.

use bevy::prelude::Entity;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use processing::prelude::{
    image, image_destroy, texture3d_create, texture3d_readback, texture3d_write,
};

use crate::error;

/// Map an FFI format code to a texture format.
///
/// 0 = r16float (2 B/texel; storage write + trilinear on Metal via
///     adapter-specific format features — the voxel T-field default),
/// 1 = r32float (4 B; storage write everywhere, NOT filterable on Apple GPUs),
/// 2 = rg16float (4 B),
/// 3 = rgba16float (8 B; base-WebGPU-safe storage + filterable fallback),
/// 4 = rgba8unorm (4 B),
/// 5 = r32uint (4 B; storage write, integer loads only).
fn texture3d_format(code: u8) -> Result<TextureFormat, error::ProcessingError> {
    Ok(match code {
        0 => TextureFormat::R16Float,
        1 => TextureFormat::R32Float,
        2 => TextureFormat::Rg16Float,
        3 => TextureFormat::Rgba16Float,
        4 => TextureFormat::Rgba8Unorm,
        5 => TextureFormat::R32Uint,
        _ => {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "unknown texture3d format code: {code}"
            )));
        }
    })
}

/// Create a zero-initialized 3D texture. Returns the texture id (usable
/// wherever an image id is accepted: shader/material texture params).
///
/// # Safety
/// - Init has been called; called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_texture3d_create(
    width: u32,
    height: u32,
    depth: u32,
    format: u8,
) -> u64 {
    error::clear_error();
    error::check(|| {
        let format = texture3d_format(format)?;
        texture3d_create(
            Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            },
            format,
        )
    })
    .map(|entity| entity.to_bits())
    .unwrap_or(0)
}

/// Bytes per texel for a format code (for sizing upload/readback buffers).
#[unsafe(no_mangle)]
pub extern "C" fn processing_texture3d_texel_size(format: u8) -> u32 {
    error::clear_error();
    error::check(|| {
        let format = texture3d_format(format)?;
        image::pixel_size(format).map(|s| s as u32)
    })
    .unwrap_or(0)
}

/// Upload a full volume of tightly-packed texel bytes (x fastest, then y,
/// then z; `data_len` must equal width*height*depth*texel_size).
///
/// # Safety
/// - `data` is valid for `data_len` bytes.
/// - Init has been called; called from the same thread as init.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_texture3d_write(
    texture_id: u64,
    data: *const u8,
    data_len: usize,
) {
    error::clear_error();
    let data = unsafe { std::slice::from_raw_parts(data, data_len) };
    error::check(|| texture3d_write(Entity::from_bits(texture_id), data.to_vec()));
}

/// Read the full volume back as tightly-packed texel bytes. `out_len` must
/// equal width*height*depth*texel_size.
///
/// # Safety
/// - `out` is valid for `out_len` writable bytes.
/// - Init has been called; called from the same thread as init.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_texture3d_readback(
    texture_id: u64,
    out: *mut u8,
    out_len: usize,
) {
    error::clear_error();
    error::check(|| {
        let bytes = texture3d_readback(Entity::from_bits(texture_id))?;
        if bytes.len() != out_len {
            let msg = format!(
                "texture3d readback: buffer size mismatch: expected {}, got {}",
                bytes.len(),
                out_len
            );
            return Err(error::ProcessingError::InvalidArgument(msg));
        }
        unsafe {
            std::slice::from_raw_parts_mut(out, out_len).copy_from_slice(&bytes);
        }
        Ok(())
    });
}

/// Destroy a 3D texture (same entity space as images).
///
/// # Safety
/// - Init has been called; called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_texture3d_destroy(texture_id: u64) {
    error::clear_error();
    error::check(|| image_destroy(Entity::from_bits(texture_id)));
}
