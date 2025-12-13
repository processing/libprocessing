use bevy::{
    prelude::Entity,
    render::render_resource::{Extent3d, TextureFormat},
};
use processing::prelude::*;

use crate::color::Color;

mod color;
mod error;

/// Initialize libProcessing.
///
/// SAFETY:
/// - This is called from the main thread if the platform requires it.
/// - This can only be called once.
#[unsafe(no_mangle)]
pub extern "C" fn processing_init() {
    error::clear_error();
    error::check(init);
}

/// Create a WebGPU surface from a macOS NSWindow handle.
///
/// SAFETY:
/// - Init has been called.
/// - window_handle is a valid NSWindow pointer.
/// - This is called from the same thread as init.
#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn processing_surface_create(
    window_handle: u64,
    _display_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> u64 {
    error::clear_error();
    error::check(|| surface_create_macos(window_handle, width, height, scale_factor))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// Create a WebGPU surface from a Windows HWND handle.
///
/// SAFETY:
/// - Init has been called.
/// - window_handle is a valid HWND.
/// - This is called from the same thread as init.
#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
pub extern "C" fn processing_surface_create(
    window_handle: u64,
    _display_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> u64 {
    error::clear_error();
    error::check(|| surface_create_windows(window_handle, width, height, scale_factor))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// Create a WebGPU surface from a Wayland window and display handle.
///
/// SAFETY:
/// - Init has been called.
/// - window_handle is a valid wl_surface pointer.
/// - display_handle is a valid wl_display pointer.
/// - This is called from the same thread as init.
#[cfg(all(target_os = "linux", feature = "wayland"))]
#[unsafe(no_mangle)]
pub extern "C" fn processing_surface_create_wayland(
    window_handle: u64,
    display_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> u64 {
    error::clear_error();
    error::check(|| {
        surface_create_wayland(window_handle, display_handle, width, height, scale_factor)
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

/// Create a WebGPU surface from an X11 window and display handle.
///
/// SAFETY:
/// - Init has been called.
/// - window_handle is a valid X11 Window ID.
/// - display_handle is a valid X11 Display pointer.
/// - This is called from the same thread as init.
#[cfg(all(target_os = "linux", feature = "x11"))]
#[unsafe(no_mangle)]
pub extern "C" fn processing_surface_create_x11(
    window_handle: u64,
    display_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> u64 {
    error::clear_error();
    error::check(|| surface_create_x11(window_handle, display_handle, width, height, scale_factor))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// Destroy the surface associated with the given window ID.
///
/// SAFETY:
/// - Init and surface_create have been called.
/// - window_id is a valid ID returned from surface_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_surface_destroy(window_id: u64) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| surface_destroy(window_entity));
}

/// Update window size when resized.
///
/// SAFETY:
/// - Init and surface_create have been called.
/// - window_id is a valid ID returned from surface_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_surface_resize(window_id: u64, width: u32, height: u32) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| surface_resize(window_entity, width, height));
}

/// Set the background color for the given window.
///
/// SAFETY:
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_background_color(window_id: u64, color: Color) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| {
        graphics_record_command(window_entity, DrawCommand::BackgroundColor(color.into()))
    });
}

/// Set the background image for the given window.
///
/// SAFETY:
/// - This is called from the same thread as init.
/// - image_id is a valid ID returned from processing_image_create.
/// - The image has been fully uploaded.
#[unsafe(no_mangle)]
pub extern "C" fn processing_background_image(window_id: u64, image_id: u64) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    let image_entity = Entity::from_bits(image_id);
    error::check(|| {
        graphics_record_command(window_entity, DrawCommand::BackgroundImage(image_entity))
    });
}

/// Begins the draw for the given window.
///
/// SAFETY:
/// - Init has been called and exit has not been called.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_begin_draw(window_id: u64) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| graphics_begin_draw(window_entity));
}

/// Flushes recorded draw commands for the given window.
///
/// SAFETY:
/// - Init has been called and exit has not been called.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_flush(window_id: u64) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| graphics_flush(window_entity));
}

/// Ends the draw for the given window and presents the frame.
///
/// SAFETY:
/// - Init has been called and exit has not been called.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_end_draw(window_id: u64) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| graphics_end_draw(window_entity));
}

/// Shuts down internal resources with given exit code, but does *not* terminate the process.
///
/// SAFETY:
/// - This is called from the same thread as init.
/// - Caller ensures that update is never called again after exit.
#[unsafe(no_mangle)]
pub extern "C" fn processing_exit(exit_code: u8) {
    error::clear_error();
    error::check(|| exit(exit_code));
}

/// Set the fill color.
///
/// SAFETY:
/// - Init and surface_create have been called.
/// - window_id is a valid ID returned from surface_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_fill(window_id: u64, r: f32, g: f32, b: f32, a: f32) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    let color = bevy::color::Color::srgba(r, g, b, a);
    error::check(|| graphics_record_command(window_entity, DrawCommand::Fill(color)));
}

/// Set the stroke color.
///
/// SAFETY:
/// - Init and surface_create have been called.
/// - window_id is a valid ID returned from surface_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_stroke_color(window_id: u64, r: f32, g: f32, b: f32, a: f32) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    let color = bevy::color::Color::srgba(r, g, b, a);
    error::check(|| graphics_record_command(window_entity, DrawCommand::StrokeColor(color)));
}

/// Set the stroke weight.
///
/// SAFETY:
/// - Init and surface_create have been called.
/// - window_id is a valid ID returned from surface_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_stroke_weight(window_id: u64, weight: f32) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| graphics_record_command(window_entity, DrawCommand::StrokeWeight(weight)));
}

/// Disable fill for subsequent shapes.
///
/// SAFETY:
/// - Init and surface_create have been called.
/// - window_id is a valid ID returned from surface_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_no_fill(window_id: u64) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| graphics_record_command(window_entity, DrawCommand::NoFill));
}

/// Disable stroke for subsequent shapes.
///
/// SAFETY:
/// - Init and surface_create have been called.
/// - window_id is a valid ID returned from surface_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_no_stroke(window_id: u64) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| graphics_record_command(window_entity, DrawCommand::NoStroke));
}

/// Draw a rectangle.
///
/// SAFETY:
/// - Init and surface_create have been called.
/// - window_id is a valid ID returned from surface_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_rect(
    window_id: u64,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    tl: f32,
    tr: f32,
    br: f32,
    bl: f32,
) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    error::check(|| {
        graphics_record_command(
            window_entity,
            DrawCommand::Rect {
                x,
                y,
                w,
                h,
                radii: [tl, tr, br, bl],
            },
        )
    });
}

/// Create an image from raw pixel data.
///
/// # Safety
/// - Init has been called.
/// - data is a valid pointer to data_len bytes of RGBA pixel data.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_image_create(
    width: u32,
    height: u32,
    data: *const u8,
    data_len: usize,
) -> u64 {
    error::clear_error();
    // SAFETY: Caller must ensure that `data` is valid for `data_len` bytes.
    let data = unsafe { std::slice::from_raw_parts(data, data_len) };
    error::check(|| {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        image_create(size, data.to_vec(), TextureFormat::Rgba8UnormSrgb)
    })
    .map(|entity| entity.to_bits())
    .unwrap_or(0)
}

/// Load an image from a file path.
///
/// # Safety
/// - Init has been called.
/// - path is a valid null-terminated C string.
/// - This is called from the same thread as init.
///
/// Note: This function is currently synchronous but Bevy's asset loading is async.
/// The image may not be immediately available. This needs to be improved.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_image_load(path: *const std::ffi::c_char) -> u64 {
    error::clear_error();

    // SAFETY: Caller guarantees path is a valid C string
    let c_str = unsafe { std::ffi::CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            error::set_error("Invalid UTF-8 in image path");
            return 0;
        }
    };

    error::check(|| image_load(path_str))
        .map(|entity| entity.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_image_resize(image_id: u64, new_width: u32, new_height: u32) {
    error::clear_error();
    let image_entity = Entity::from_bits(image_id);
    let new_size = Extent3d {
        width: new_width,
        height: new_height,
        depth_or_array_layers: 1,
    };
    error::check(|| image_resize(image_entity, new_size));
}

/// Load pixels from an image into a caller-provided buffer.
///
/// # Safety
/// - Init and image_create have been called.
/// - image_id is a valid ID returned from image_create.
/// - buffer is a valid pointer to at least buffer_len Color elements.
/// - buffer_len must equal width * height of the image.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_image_readback(
    image_id: u64,
    buffer: *mut Color,
    buffer_len: usize,
) {
    error::clear_error();
    let image_entity = Entity::from_bits(image_id);
    error::check(|| {
        let colors = image_readback(image_entity)?;

        // Validate buffer size
        if colors.len() != buffer_len {
            let error_msg = format!(
                "Buffer size mismatch: expected {}, got {}",
                colors.len(),
                buffer_len
            );
            error::set_error(&error_msg);
            return Err(error::ProcessingError::InvalidArgument(error_msg));
        }

        // SAFETY: Caller guarantees buffer is valid for buffer_len elements
        unsafe {
            let buffer_slice = std::slice::from_raw_parts_mut(buffer, buffer_len);
            for (i, color) in colors.iter().enumerate() {
                buffer_slice[i] = Color::from(*color);
            }
        }

        Ok(())
    });
}
