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
    error::check(|| init(Config::default()));
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

/// Create a graphics context for a surface.
///
/// SAFETY:
/// - Init and surface_create have been called.
/// - surface_id is a valid ID returned from surface_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_graphics_create(surface_id: u64, width: u32, height: u32) -> u64 {
    error::clear_error();
    let surface_entity = Entity::from_bits(surface_id);
    error::check(|| graphics_create(surface_entity, width, height))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// Destroy a graphics context.
///
/// SAFETY:
/// - Init and graphics_create have been called.
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_graphics_destroy(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_destroy(graphics_entity));
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

/// Set the background color for the given graphics context.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_background_color(graphics_id: u64, color: Color) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::BackgroundColor(color.into()))
    });
}

/// Set the background image for the given graphics context.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - image_id is a valid ID returned from processing_image_create.
/// - The image has been fully uploaded.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_background_image(graphics_id: u64, image_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let image_entity = Entity::from_bits(image_id);
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::BackgroundImage(image_entity))
    });
}

/// Begins the draw for the given graphics context.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - Init has been called and exit has not been called.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_begin_draw(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_begin_draw(graphics_entity));
}

/// Flushes recorded draw commands for the given graphics context.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - Init has been called and exit has not been called.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_flush(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_flush(graphics_entity));
}

/// Ends the draw for the given graphics context and presents the frame.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - Init has been called and exit has not been called.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_end_draw(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_end_draw(graphics_entity));
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
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_fill(graphics_id: u64, r: f32, g: f32, b: f32, a: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let color = bevy::color::Color::srgba(r, g, b, a);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Fill(color)));
}

/// Set the stroke color.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_stroke_color(graphics_id: u64, r: f32, g: f32, b: f32, a: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let color = bevy::color::Color::srgba(r, g, b, a);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::StrokeColor(color)));
}

/// Set the stroke weight.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_stroke_weight(graphics_id: u64, weight: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::StrokeWeight(weight)));
}

/// Disable fill for subsequent shapes.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_no_fill(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::NoFill));
}

/// Disable stroke for subsequent shapes.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_no_stroke(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::NoStroke));
}

/// Push the current transformation matrix onto the stack.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_push_matrix(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::PushMatrix));
}

/// Pop the transformation matrix from the stack.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_pop_matrix(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::PopMatrix));
}

/// Reset the transformation matrix to identity.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_reset_matrix(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::ResetMatrix));
}

/// Translate the coordinate system.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_translate(graphics_id: u64, x: f32, y: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Translate { x, y }));
}

/// Rotate the coordinate system.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_rotate(graphics_id: u64, angle: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Rotate { angle }));
}

/// Scale the coordinate system.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_scale(graphics_id: u64, x: f32, y: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Scale { x, y }));
}

/// Shear along the X axis.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_shear_x(graphics_id: u64, angle: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::ShearX { angle }));
}

/// Shear along the Y axis.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_shear_y(graphics_id: u64, angle: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::ShearY { angle }));
}

/// Draw a rectangle.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_rect(
    graphics_id: u64,
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
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
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

#[unsafe(no_mangle)]
pub extern "C" fn processing_mode_3d(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_mode_3d(graphics_entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_mode_2d(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_mode_2d(graphics_entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_perspective(
    graphics_id: u64,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_perspective(graphics_entity, fov, aspect, near, far));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_ortho(
    graphics_id: u64,
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_ortho(graphics_entity, left, right, bottom, top, near, far));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_set_position(entity_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_set_position(entity, x, y, z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_translate(entity_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_translate(entity, x, y, z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_set_rotation(entity_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_set_rotation(entity, x, y, z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_rotate_x(entity_id: u64, angle: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_rotate_x(entity, angle));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_rotate_y(entity_id: u64, angle: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_rotate_y(entity, angle));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_rotate_z(entity_id: u64, angle: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_rotate_z(entity, angle));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_rotate_axis(
    entity_id: u64,
    angle: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_rotate_axis(entity, angle, axis_x, axis_y, axis_z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_set_scale(entity_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_set_scale(entity, x, y, z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_scale(entity_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_scale(entity, x, y, z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_look_at(
    entity_id: u64,
    target_x: f32,
    target_y: f32,
    target_z: f32,
) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_look_at(entity, target_x, target_y, target_z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_reset(entity_id: u64) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_reset(entity));
}

pub const PROCESSING_ATTR_FORMAT_FLOAT: u8 = 1;
pub const PROCESSING_ATTR_FORMAT_FLOAT2: u8 = 2;
pub const PROCESSING_ATTR_FORMAT_FLOAT3: u8 = 3;
pub const PROCESSING_ATTR_FORMAT_FLOAT4: u8 = 4;

pub const PROCESSING_TOPOLOGY_POINT_LIST: u8 = 0;
pub const PROCESSING_TOPOLOGY_LINE_LIST: u8 = 1;
pub const PROCESSING_TOPOLOGY_LINE_STRIP: u8 = 2;
pub const PROCESSING_TOPOLOGY_TRIANGLE_LIST: u8 = 3;
pub const PROCESSING_TOPOLOGY_TRIANGLE_STRIP: u8 = 4;

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_layout_create() -> u64 {
    error::clear_error();
    error::check(geometry_layout_create)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_layout_add_position(layout_id: u64) {
    error::clear_error();
    let entity = Entity::from_bits(layout_id);
    error::check(|| geometry_layout_add_position(entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_layout_add_normal(layout_id: u64) {
    error::clear_error();
    let entity = Entity::from_bits(layout_id);
    error::check(|| geometry_layout_add_normal(entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_layout_add_color(layout_id: u64) {
    error::clear_error();
    let entity = Entity::from_bits(layout_id);
    error::check(|| geometry_layout_add_color(entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_layout_add_uv(layout_id: u64) {
    error::clear_error();
    let entity = Entity::from_bits(layout_id);
    error::check(|| geometry_layout_add_uv(entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_layout_add_attribute(layout_id: u64, attr_id: u64) {
    error::clear_error();
    let layout_entity = Entity::from_bits(layout_id);
    let attr_entity = Entity::from_bits(attr_id);
    error::check(|| geometry_layout_add_attribute(layout_entity, attr_entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_layout_destroy(layout_id: u64) {
    error::clear_error();
    let entity = Entity::from_bits(layout_id);
    error::check(|| geometry_layout_destroy(entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_create_with_layout(layout_id: u64, topology: u8) -> u64 {
    error::clear_error();
    let Some(topo) = geometry::Topology::from_u8(topology) else {
        error::set_error("Invalid topology");
        return 0;
    };
    let entity = Entity::from_bits(layout_id);
    error::check(|| geometry_create_with_layout(entity, topo))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_create(topology: u8) -> u64 {
    error::clear_error();
    let Some(topo) = geometry::Topology::from_u8(topology) else {
        error::set_error("Invalid topology");
        return 0;
    };
    error::check(|| geometry_create(topo))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_normal(geo_id: u64, nx: f32, ny: f32, nz: f32) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_normal(entity, nx, ny, nz));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_color(geo_id: u64, r: f32, g: f32, b: f32, a: f32) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_color(entity, r, g, b, a));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_uv(geo_id: u64, u: f32, v: f32) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_uv(entity, u, v));
}

/// # Safety
/// - `name` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_geometry_attribute_create(
    name: *const std::ffi::c_char,
    format: u8,
) -> u64 {
    error::clear_error();

    let c_str = unsafe { std::ffi::CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            error::set_error("Invalid UTF-8 in attribute name");
            return 0;
        }
    };

    let attr_format = match geometry::AttributeFormat::from_u8(format) {
        Some(f) => f,
        None => {
            error::set_error("Invalid attribute format");
            return 0;
        }
    };

    error::check(|| geometry_attribute_create(name_str, attr_format))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_destroy(attr_id: u64) {
    error::clear_error();
    let entity = Entity::from_bits(attr_id);
    error::check(|| geometry_attribute_destroy(entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_position() -> u64 {
    geometry_attribute_position().to_bits()
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_normal() -> u64 {
    geometry_attribute_normal().to_bits()
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_color() -> u64 {
    geometry_attribute_color().to_bits()
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_uv() -> u64 {
    geometry_attribute_uv().to_bits()
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_float(geo_id: u64, attr_id: u64, v: f32) {
    error::clear_error();
    let geo_entity = Entity::from_bits(geo_id);
    let attr_entity = Entity::from_bits(attr_id);
    error::check(|| geometry_attribute_float(geo_entity, attr_entity, v));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_float2(geo_id: u64, attr_id: u64, x: f32, y: f32) {
    error::clear_error();
    let geo_entity = Entity::from_bits(geo_id);
    let attr_entity = Entity::from_bits(attr_id);
    error::check(|| geometry_attribute_float2(geo_entity, attr_entity, x, y));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_float3(
    geo_id: u64,
    attr_id: u64,
    x: f32,
    y: f32,
    z: f32,
) {
    error::clear_error();
    let geo_entity = Entity::from_bits(geo_id);
    let attr_entity = Entity::from_bits(attr_id);
    error::check(|| geometry_attribute_float3(geo_entity, attr_entity, x, y, z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_float4(
    geo_id: u64,
    attr_id: u64,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) {
    error::clear_error();
    let geo_entity = Entity::from_bits(geo_id);
    let attr_entity = Entity::from_bits(attr_id);
    error::check(|| geometry_attribute_float4(geo_entity, attr_entity, x, y, z, w));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_vertex(geo_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_vertex(entity, x, y, z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_index(geo_id: u64, i: u32) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_index(entity, i));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_vertex_count(geo_id: u64) -> u32 {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_vertex_count(entity)).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_index_count(geo_id: u64) -> u32 {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_index_count(entity)).unwrap_or(0)
}

/// # Safety
/// - `out` must be valid for writes of `out_len` elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_geometry_get_positions(
    geo_id: u64,
    start: u32,
    end: u32,
    out: *mut [f32; 3],
    out_len: u32,
) -> u32 {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    let positions = error::check(|| geometry_get_positions(entity, start as usize, end as usize));
    match positions {
        Some(p) => {
            let count = p.len().min(out_len as usize);
            unsafe { std::ptr::copy_nonoverlapping(p.as_ptr(), out, count) };
            count as u32
        }
        None => 0,
    }
}

/// # Safety
/// - `out` must be valid for writes of `out_len` elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_geometry_get_normals(
    geo_id: u64,
    start: u32,
    end: u32,
    out: *mut [f32; 3],
    out_len: u32,
) -> u32 {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    let normals = error::check(|| geometry_get_normals(entity, start as usize, end as usize));
    match normals {
        Some(n) => {
            let count = n.len().min(out_len as usize);
            unsafe { std::ptr::copy_nonoverlapping(n.as_ptr(), out, count) };
            count as u32
        }
        None => 0,
    }
}

/// # Safety
/// - `out` must be valid for writes of `out_len` elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_geometry_get_colors(
    geo_id: u64,
    start: u32,
    end: u32,
    out: *mut [f32; 4],
    out_len: u32,
) -> u32 {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    let colors = error::check(|| geometry_get_colors(entity, start as usize, end as usize));
    match colors {
        Some(c) => {
            let count = c.len().min(out_len as usize);
            unsafe { std::ptr::copy_nonoverlapping(c.as_ptr(), out, count) };
            count as u32
        }
        None => 0,
    }
}

/// # Safety
/// - `out` must be valid for writes of `out_len` elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_geometry_get_uvs(
    geo_id: u64,
    start: u32,
    end: u32,
    out: *mut [f32; 2],
    out_len: u32,
) -> u32 {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    let uvs = error::check(|| geometry_get_uvs(entity, start as usize, end as usize));
    match uvs {
        Some(u) => {
            let count = u.len().min(out_len as usize);
            unsafe { std::ptr::copy_nonoverlapping(u.as_ptr(), out, count) };
            count as u32
        }
        None => 0,
    }
}

/// # Safety
/// - `out` must be valid for writes of `out_len` elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_geometry_get_indices(
    geo_id: u64,
    start: u32,
    end: u32,
    out: *mut u32,
    out_len: u32,
) -> u32 {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    let indices = error::check(|| geometry_get_indices(entity, start as usize, end as usize));
    match indices {
        Some(i) => {
            let count = i.len().min(out_len as usize);
            unsafe { std::ptr::copy_nonoverlapping(i.as_ptr(), out, count) };
            count as u32
        }
        None => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_set_vertex(geo_id: u64, index: u32, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_set_vertex(entity, index, x, y, z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_set_normal(
    geo_id: u64,
    index: u32,
    nx: f32,
    ny: f32,
    nz: f32,
) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_set_normal(entity, index, nx, ny, nz));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_set_color(
    geo_id: u64,
    index: u32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_set_color(entity, index, r, g, b, a));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_set_uv(geo_id: u64, index: u32, u: f32, v: f32) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_set_uv(entity, index, u, v));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_destroy(geo_id: u64) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_destroy(entity));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_model(graphics_id: u64, geo_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let geo_entity = Entity::from_bits(geo_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Geometry(geo_entity)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_box(width: f32, height: f32, depth: f32) -> u64 {
    error::clear_error();
    error::check(|| geometry_box(width, height, depth))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_light_create_directional(
    graphics_id: u64,
    color: Color,
    illuminance: f32,
) -> u64 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| light_create_directional(graphics_entity, color.into(), illuminance))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_light_create_point(
    graphics_id: u64,
    color: Color,
    intensity: f32,
    range: f32,
    radius: f32,
) -> u64 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| light_create_point(graphics_entity, color.into(), intensity, range, radius))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_light_create_spot(
    graphics_id: u64,
    color: Color,
    intensity: f32,
    range: f32,
    radius: f32,
    inner_angle: f32,
    outer_angle: f32,
) -> u64 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        light_create_spot(
            graphics_entity,
            color.into(),
            intensity,
            range,
            radius,
            inner_angle,
            outer_angle,
        )
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}
