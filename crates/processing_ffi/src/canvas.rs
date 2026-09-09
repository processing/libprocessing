//! Offscreen canvases, state (accumulator) canvases, and canvas compositing.
//!
//! A **state canvas** is the feedback/simulation counterpart to a picture
//! canvas: float format, no bloom, no tonemapping, and LINEAR color writes.
//! Post-processing writes back through the view target, so on a canvas whose
//! pixels are data (a reaction-diffusion field, an accumulator) it silently
//! re-transforms the state every frame; and sRGB color resolution means
//! `fill(0.5)` would store 0.21. Both are fatal for patch-style work and both
//! are off by construction here.

use bevy::prelude::Entity;
use bevy::render::render_resource::TextureFormat;
use processing::prelude::{
    filter_composite, filter_feedback, filter_set, graphics_apply_filter, graphics_create_ex,
    graphics_flush, shader_value::ShaderValue, surface_create_offscreen,
};

use crate::error;

/// Composite mode: REPLACE — Processing's `copy()` (filters/composite.wgsl).
pub const COMPOSITE_MODE_REPLACE: u32 = 9;
/// Composite mode: MASK — Processing's `mask()`.
pub const COMPOSITE_MODE_MASK: u32 = 10;

fn texture_format(hdr: bool) -> TextureFormat {
    if hdr {
        TextureFormat::Rgba16Float
    } else {
        TextureFormat::Rgba8UnormSrgb
    }
}

/// Create an offscreen render surface (an image-backed target, no window).
/// Pair with `processing_graphics_create_ex` to get a drawable canvas.
/// Returns the surface entity id, or 0 on error.
///
/// SAFETY:
/// - Init has been called.
/// - Called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_surface_create_offscreen(
    width: u32,
    height: u32,
    scale_factor: f32,
    hdr: bool,
) -> u64 {
    error::clear_error();
    error::check(|| surface_create_offscreen(width, height, scale_factor, texture_format(hdr)))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// Create a graphics context with explicit format and state semantics.
///
/// `hdr`: float (Rgba16Float) vs Rgba8UnormSrgb target.
/// `state`: when true the canvas is a STATE canvas — post-process-free
/// (bloom/tonemapping never applied, and `bloom()` on it is ignored with a
/// warning) and linear-color, so drawn values round-trip exactly.
/// Returns the graphics entity id, or 0 on error.
#[unsafe(no_mangle)]
pub extern "C" fn processing_graphics_create_ex(
    surface_id: u64,
    width: u32,
    height: u32,
    hdr: bool,
    state: bool,
) -> u64 {
    error::clear_error();
    let surface_entity = Entity::from_bits(surface_id);
    error::check(|| {
        graphics_create_ex(
            surface_entity,
            width,
            height,
            texture_format(hdr),
            state,
        )
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

/// Composite a texture-bearing source onto a graphics target through the
/// built-in composite filter — the way to get an offscreen buffer onto the
/// main canvas.
///
/// `src_id` is the texture entity: an image, or an offscreen graphics'
/// SURFACE entity. `src_graphics_id` is that graphics (or 0), flushed first so
/// the latest content is what gets sampled. `src_rect`/`dst_rect` are uv-space
/// `[x0, y0, x1, y1]`, or null for the full extent.
///
/// SAFETY:
/// - Init has been called.
/// - `src_rect`/`dst_rect` are null or valid for 4 f32 reads.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_graphics_composite(
    dst_graphics_id: u64,
    src_id: u64,
    src_graphics_id: u64,
    mode: u32,
    src_rect: *const f32,
    dst_rect: *const f32,
    opacity: f32,
) {
    error::clear_error();
    let read_rect = |p: *const f32| -> [f32; 4] {
        if p.is_null() {
            [0.0, 0.0, 1.0, 1.0]
        } else {
            let s = unsafe { std::slice::from_raw_parts(p, 4) };
            [s[0], s[1], s[2], s[3]]
        }
    };
    let src_rect = read_rect(src_rect);
    let dst_rect = read_rect(dst_rect);
    error::check(|| {
        if src_graphics_id != 0 {
            graphics_flush(Entity::from_bits(src_graphics_id))?;
        }
        let filter = filter_composite()?;
        filter_set(
            filter,
            "src",
            ShaderValue::Texture(Entity::from_bits(src_id)),
        )?;
        filter_set(filter, "mode", ShaderValue::UInt(mode))?;
        filter_set(filter, "opacity", ShaderValue::Float(opacity))?;
        filter_set(filter, "src_rect", ShaderValue::Float4(src_rect))?;
        filter_set(filter, "dst_rect", ShaderValue::Float4(dst_rect))?;
        graphics_apply_filter(Entity::from_bits(dst_graphics_id), filter)
    });
}

/// Built-in feedback pass: re-sample this graphics' previous frame through a
/// zoom/rotate/offset transform scaled by `decay`, writing it back. On a
/// canvas you never clear, call at the start of the frame and draw new content
/// on top to get feedback trails.
#[unsafe(no_mangle)]
pub extern "C" fn processing_graphics_feedback(
    graphics_id: u64,
    decay: f32,
    zoom: f32,
    angle: f32,
    offset_x: f32,
    offset_y: f32,
) {
    error::clear_error();
    error::check(|| {
        let filter = filter_feedback()?;
        filter_set(filter, "decay", ShaderValue::Float(decay))?;
        filter_set(filter, "zoom", ShaderValue::Float(zoom))?;
        filter_set(filter, "angle", ShaderValue::Float(angle))?;
        filter_set(
            filter,
            "offset",
            ShaderValue::Float2([offset_x, offset_y]),
        )?;
        graphics_apply_filter(Entity::from_bits(graphics_id), filter)
    });
}

/// The built-in feedback filter entity (for direct `filter_set` tuning).
/// Returns 0 on error.
#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_feedback() -> u64 {
    error::clear_error();
    error::check(filter_feedback)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// The built-in composite filter entity. Returns 0 on error.
#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_composite() -> u64 {
    error::clear_error();
    error::check(filter_composite)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}
