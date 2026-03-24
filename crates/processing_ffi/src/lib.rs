use bevy::{
    math::{Vec2, Vec3, Vec4},
    prelude::Entity,
    render::render_resource::{Extent3d, TextureFormat},
};
use processing::prelude::{error::ProcessingError, *};

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
    error::check(|| graphics_create(surface_entity, width, height, TextureFormat::Rgba16Float))
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
        let mode = graphics_get_color_mode(graphics_entity)?;
        let color = color.resolve(&mode);
        graphics_record_command(graphics_entity, DrawCommand::BackgroundColor(color))
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

/// Set the color mode for a graphics context.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_color_mode(
    graphics_id: u64,
    space: u8,
    max1: f32,
    max2: f32,
    max3: f32,
    max_alpha: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        let space = processing::prelude::color::ColorSpace::from_u8(space).ok_or_else(|| {
            processing::prelude::error::ProcessingError::InvalidArgument(format!(
                "unknown color space: {space}"
            ))
        })?;
        let mode = processing::prelude::color::ColorMode::new(space, max1, max2, max3, max_alpha);
        graphics_set_color_mode(graphics_entity, mode)
    });
}

/// Set the fill color.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_fill(graphics_id: u64, color: Color) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        graphics_record_command(graphics_entity, DrawCommand::Fill(color.resolve(&mode)))
    });
}

/// Set the stroke color.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_stroke_color(graphics_id: u64, color: Color) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        graphics_record_command(
            graphics_entity,
            DrawCommand::StrokeColor(color.resolve(&mode)),
        )
    });
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

/// Set the stroke cap mode.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_stroke_cap(graphics_id: u64, cap: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::StrokeCap(processing::prelude::StrokeCapMode::from(cap)),
        )
    });
}

/// Set the stroke join mode.
#[unsafe(no_mangle)]
pub extern "C" fn processing_set_stroke_join(graphics_id: u64, join: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::StrokeJoin(processing::prelude::StrokeJoinMode::from(join)),
        )
    });
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
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::Translate(Vec2::new(x, y)))
    });
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
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Scale(Vec2::new(x, y))));
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
                buffer_slice[i] = Color::from_linear(*color);
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
    error::check(|| {
        graphics_perspective(
            graphics_entity,
            fov,
            aspect,
            near,
            far,
            bevy::math::Vec4::new(0.0, 0.0, -1.0, -near),
        )
    });
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
    error::check(|| transform_set_position(entity, Vec3::new(x, y, z)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_translate(entity_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_translate(entity, Vec3::new(x, y, z)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_set_rotation(entity_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_set_rotation(entity, Vec3::new(x, y, z)));
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
    error::check(|| transform_rotate_axis(entity, angle, Vec3::new(axis_x, axis_y, axis_z)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_set_scale(entity_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_set_scale(entity, Vec3::new(x, y, z)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_transform_scale(entity_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let entity = Entity::from_bits(entity_id);
    error::check(|| transform_scale(entity, Vec3::new(x, y, z)));
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
    error::check(|| transform_look_at(entity, Vec3::new(target_x, target_y, target_z)));
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

pub const PROCESSING_STROKE_CAP_ROUND: u8 = 0;
pub const PROCESSING_STROKE_CAP_SQUARE: u8 = 1;
pub const PROCESSING_STROKE_CAP_PROJECT: u8 = 2;

pub const PROCESSING_STROKE_JOIN_ROUND: u8 = 0;
pub const PROCESSING_STROKE_JOIN_MITER: u8 = 1;
pub const PROCESSING_STROKE_JOIN_BEVEL: u8 = 2;

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
    error::check(|| geometry_normal(entity, Vec3::new(nx, ny, nz)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_color(geo_id: u64, r: f32, g: f32, b: f32, a: f32) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_color(entity, Vec4::new(r, g, b, a)));
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
    error::check(|| geometry_vertex(entity, Vec3::new(x, y, z)));
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
    error::check(|| geometry_set_vertex(entity, index, Vec3::new(x, y, z)));
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
    error::check(|| geometry_set_normal(entity, index, Vec3::new(nx, ny, nz)));
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
    error::check(|| geometry_set_color(entity, index, Vec4::new(r, g, b, a)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_set_uv(geo_id: u64, index: u32, u: f32, v: f32) {
    error::clear_error();
    let entity = Entity::from_bits(geo_id);
    error::check(|| geometry_set_uv(entity, index, Vec2::new(u, v)));
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
pub extern "C" fn processing_geometry_sphere(radius: f32, sectors: u32, stacks: u32) -> u64 {
    error::clear_error();
    error::check(|| geometry_sphere(radius, sectors, stacks))
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
    error::check(|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        light_create_directional(graphics_entity, color.resolve(&mode), illuminance)
    })
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
    error::check(|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        light_create_point(
            graphics_entity,
            color.resolve(&mode),
            intensity,
            range,
            radius,
        )
    })
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
        let mode = graphics_get_color_mode(graphics_entity)?;
        light_create_spot(
            graphics_entity,
            color.resolve(&mode),
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

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_create_pbr() -> u64 {
    error::clear_error();
    error::check(material_create_pbr)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// Set float value for `name` field on Material.
///
/// # Safety
/// - `name` must be non-null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_material_set_float(
    mat_id: u64,
    name: *const std::ffi::c_char,
    value: f32,
) {
    error::clear_error();
    let name = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().unwrap();
    error::check(|| {
        material_set(
            Entity::from_bits(mat_id),
            name,
            material::MaterialValue::Float(value),
        )
    });
}

/// Set float4 value for `name` field on Material.
///
/// # Safety
/// - `name` must be non-null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_material_set_float4(
    mat_id: u64,
    name: *const std::ffi::c_char,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    error::clear_error();
    let name = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().unwrap();
    error::check(|| {
        material_set(
            Entity::from_bits(mat_id),
            name,
            material::MaterialValue::Float4([r, g, b, a]),
        )
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_destroy(mat_id: u64) {
    error::clear_error();
    error::check(|| material_destroy(Entity::from_bits(mat_id)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_material(window_id: u64, mat_id: u64) {
    error::clear_error();
    let window_entity = Entity::from_bits(window_id);
    let mat_entity = Entity::from_bits(mat_id);
    error::check(|| graphics_record_command(window_entity, DrawCommand::Material(mat_entity)));
}

// Mouse buttons
pub const PROCESSING_MOUSE_LEFT: u8 = 0;
pub const PROCESSING_MOUSE_MIDDLE: u8 = 1;
pub const PROCESSING_MOUSE_RIGHT: u8 = 2;

// Key codes (GLFW values)
pub const PROCESSING_KEY_SPACE: u32 = 32;
pub const PROCESSING_KEY_QUOTE: u32 = 39;
pub const PROCESSING_KEY_COMMA: u32 = 44;
pub const PROCESSING_KEY_MINUS: u32 = 45;
pub const PROCESSING_KEY_PERIOD: u32 = 46;
pub const PROCESSING_KEY_SLASH: u32 = 47;
pub const PROCESSING_KEY_0: u32 = 48;
pub const PROCESSING_KEY_1: u32 = 49;
pub const PROCESSING_KEY_2: u32 = 50;
pub const PROCESSING_KEY_3: u32 = 51;
pub const PROCESSING_KEY_4: u32 = 52;
pub const PROCESSING_KEY_5: u32 = 53;
pub const PROCESSING_KEY_6: u32 = 54;
pub const PROCESSING_KEY_7: u32 = 55;
pub const PROCESSING_KEY_8: u32 = 56;
pub const PROCESSING_KEY_9: u32 = 57;
pub const PROCESSING_KEY_SEMICOLON: u32 = 59;
pub const PROCESSING_KEY_EQUAL: u32 = 61;
pub const PROCESSING_KEY_A: u32 = 65;
pub const PROCESSING_KEY_B: u32 = 66;
pub const PROCESSING_KEY_C: u32 = 67;
pub const PROCESSING_KEY_D: u32 = 68;
pub const PROCESSING_KEY_E: u32 = 69;
pub const PROCESSING_KEY_F: u32 = 70;
pub const PROCESSING_KEY_G: u32 = 71;
pub const PROCESSING_KEY_H: u32 = 72;
pub const PROCESSING_KEY_I: u32 = 73;
pub const PROCESSING_KEY_J: u32 = 74;
pub const PROCESSING_KEY_K: u32 = 75;
pub const PROCESSING_KEY_L: u32 = 76;
pub const PROCESSING_KEY_M: u32 = 77;
pub const PROCESSING_KEY_N: u32 = 78;
pub const PROCESSING_KEY_O: u32 = 79;
pub const PROCESSING_KEY_P: u32 = 80;
pub const PROCESSING_KEY_Q: u32 = 81;
pub const PROCESSING_KEY_R: u32 = 82;
pub const PROCESSING_KEY_S: u32 = 83;
pub const PROCESSING_KEY_T: u32 = 84;
pub const PROCESSING_KEY_U: u32 = 85;
pub const PROCESSING_KEY_V: u32 = 86;
pub const PROCESSING_KEY_W: u32 = 87;
pub const PROCESSING_KEY_X: u32 = 88;
pub const PROCESSING_KEY_Y: u32 = 89;
pub const PROCESSING_KEY_Z: u32 = 90;
pub const PROCESSING_KEY_BRACKET_LEFT: u32 = 91;
pub const PROCESSING_KEY_BACKSLASH: u32 = 92;
pub const PROCESSING_KEY_BRACKET_RIGHT: u32 = 93;
pub const PROCESSING_KEY_BACKQUOTE: u32 = 96;
pub const PROCESSING_KEY_ESCAPE: u32 = 256;
pub const PROCESSING_KEY_ENTER: u32 = 257;
pub const PROCESSING_KEY_TAB: u32 = 258;
pub const PROCESSING_KEY_BACKSPACE: u32 = 259;
pub const PROCESSING_KEY_INSERT: u32 = 260;
pub const PROCESSING_KEY_DELETE: u32 = 261;
pub const PROCESSING_KEY_RIGHT: u32 = 262;
pub const PROCESSING_KEY_LEFT: u32 = 263;
pub const PROCESSING_KEY_DOWN: u32 = 264;
pub const PROCESSING_KEY_UP: u32 = 265;
pub const PROCESSING_KEY_PAGE_UP: u32 = 266;
pub const PROCESSING_KEY_PAGE_DOWN: u32 = 267;
pub const PROCESSING_KEY_HOME: u32 = 268;
pub const PROCESSING_KEY_END: u32 = 269;
pub const PROCESSING_KEY_CAPS_LOCK: u32 = 280;
pub const PROCESSING_KEY_SCROLL_LOCK: u32 = 281;
pub const PROCESSING_KEY_NUM_LOCK: u32 = 282;
pub const PROCESSING_KEY_PRINT_SCREEN: u32 = 283;
pub const PROCESSING_KEY_PAUSE: u32 = 284;
pub const PROCESSING_KEY_F1: u32 = 290;
pub const PROCESSING_KEY_F2: u32 = 291;
pub const PROCESSING_KEY_F3: u32 = 292;
pub const PROCESSING_KEY_F4: u32 = 293;
pub const PROCESSING_KEY_F5: u32 = 294;
pub const PROCESSING_KEY_F6: u32 = 295;
pub const PROCESSING_KEY_F7: u32 = 296;
pub const PROCESSING_KEY_F8: u32 = 297;
pub const PROCESSING_KEY_F9: u32 = 298;
pub const PROCESSING_KEY_F10: u32 = 299;
pub const PROCESSING_KEY_F11: u32 = 300;
pub const PROCESSING_KEY_F12: u32 = 301;
pub const PROCESSING_KEY_NUMPAD_0: u32 = 320;
pub const PROCESSING_KEY_NUMPAD_1: u32 = 321;
pub const PROCESSING_KEY_NUMPAD_2: u32 = 322;
pub const PROCESSING_KEY_NUMPAD_3: u32 = 323;
pub const PROCESSING_KEY_NUMPAD_4: u32 = 324;
pub const PROCESSING_KEY_NUMPAD_5: u32 = 325;
pub const PROCESSING_KEY_NUMPAD_6: u32 = 326;
pub const PROCESSING_KEY_NUMPAD_7: u32 = 327;
pub const PROCESSING_KEY_NUMPAD_8: u32 = 328;
pub const PROCESSING_KEY_NUMPAD_9: u32 = 329;
pub const PROCESSING_KEY_NUMPAD_DECIMAL: u32 = 330;
pub const PROCESSING_KEY_NUMPAD_DIVIDE: u32 = 331;
pub const PROCESSING_KEY_NUMPAD_MULTIPLY: u32 = 332;
pub const PROCESSING_KEY_NUMPAD_SUBTRACT: u32 = 333;
pub const PROCESSING_KEY_NUMPAD_ADD: u32 = 334;
pub const PROCESSING_KEY_NUMPAD_ENTER: u32 = 335;
pub const PROCESSING_KEY_NUMPAD_EQUAL: u32 = 336;
pub const PROCESSING_KEY_SHIFT_LEFT: u32 = 340;
pub const PROCESSING_KEY_CONTROL_LEFT: u32 = 341;
pub const PROCESSING_KEY_ALT_LEFT: u32 = 342;
pub const PROCESSING_KEY_SUPER_LEFT: u32 = 343;
pub const PROCESSING_KEY_SHIFT_RIGHT: u32 = 344;
pub const PROCESSING_KEY_CONTROL_RIGHT: u32 = 345;
pub const PROCESSING_KEY_ALT_RIGHT: u32 = 346;
pub const PROCESSING_KEY_SUPER_RIGHT: u32 = 347;
pub const PROCESSING_KEY_CONTEXT_MENU: u32 = 348;

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_mouse_move(surface_id: u64, x: f32, y: f32) {
    error::clear_error();
    error::check(|| input_set_mouse_move(Entity::from_bits(surface_id), x, y));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_mouse_button(surface_id: u64, button: u8, pressed: bool) {
    error::clear_error();
    error::check(|| {
        let btn = match button {
            PROCESSING_MOUSE_LEFT => MouseButton::Left,
            PROCESSING_MOUSE_MIDDLE => MouseButton::Middle,
            PROCESSING_MOUSE_RIGHT => MouseButton::Right,
            _ => {
<<<<<<< HEAD
<<<<<<< HEAD
                return Err(ProcessingError::InvalidArgument(format!(
                    "invalid mouse button: {button}"
                )));
=======
                return Err(
                    ProcessingError::InvalidArgument(format!(
                        "invalid mouse button: {button}"
                    )),
                );
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
=======
                return Err(ProcessingError::InvalidArgument(format!(
                    "invalid mouse button: {button}"
                )));
>>>>>>> 2715c70 (Fmt.)
            }
        };
        input_set_mouse_button(Entity::from_bits(surface_id), btn, pressed)
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_scroll(surface_id: u64, x: f32, y: f32) {
    error::clear_error();
    error::check(|| input_set_scroll(Entity::from_bits(surface_id), x, y));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_key(surface_id: u64, key_code: u32, pressed: bool) {
    error::clear_error();
    error::check(|| {
        let kc = key_code_from_u32(key_code)?;
        input_set_key(Entity::from_bits(surface_id), kc, pressed)
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_char(surface_id: u64, key_code: u32, codepoint: u32) {
    error::clear_error();
    error::check(|| {
        let kc = key_code_from_u32(key_code)?;
        let ch = char::from_u32(codepoint).ok_or_else(|| {
<<<<<<< HEAD
<<<<<<< HEAD
            ProcessingError::InvalidArgument(format!("invalid codepoint: {codepoint}"))
=======
            ProcessingError::InvalidArgument(format!(
                "invalid codepoint: {codepoint}"
            ))
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
=======
            ProcessingError::InvalidArgument(format!("invalid codepoint: {codepoint}"))
>>>>>>> 2715c70 (Fmt.)
        })?;
        input_set_char(Entity::from_bits(surface_id), kc, ch)
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_cursor_enter(surface_id: u64) {
    error::clear_error();
    error::check(|| input_set_cursor_enter(Entity::from_bits(surface_id)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_cursor_leave(surface_id: u64) {
    error::clear_error();
    error::check(|| input_set_cursor_leave(Entity::from_bits(surface_id)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_focus(surface_id: u64, focused: bool) {
    error::clear_error();
    error::check(|| input_set_focus(Entity::from_bits(surface_id), focused));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_flush() {
    error::clear_error();
<<<<<<< HEAD
    error::check(input_flush);
=======
    error::check(|| input_flush());
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_mouse_x(surface_id: u64) -> f32 {
    error::clear_error();
    error::check(|| input_mouse_x(Entity::from_bits(surface_id))).unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_mouse_y(surface_id: u64) -> f32 {
    error::clear_error();
    error::check(|| input_mouse_y(Entity::from_bits(surface_id))).unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_pmouse_x(surface_id: u64) -> f32 {
    error::clear_error();
    error::check(|| input_pmouse_x(Entity::from_bits(surface_id))).unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_pmouse_y(surface_id: u64) -> f32 {
    error::clear_error();
    error::check(|| input_pmouse_y(Entity::from_bits(surface_id))).unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_mouse_is_pressed() -> bool {
    error::clear_error();
<<<<<<< HEAD
    error::check(input_mouse_is_pressed).unwrap_or(false)
=======
    error::check(|| input_mouse_is_pressed()).unwrap_or(false)
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_mouse_button() -> i8 {
    error::clear_error();
    error::check(|| {
        input_mouse_button().map(|opt| match opt {
            Some(MouseButton::Left) => PROCESSING_MOUSE_LEFT as i8,
            Some(MouseButton::Middle) => PROCESSING_MOUSE_MIDDLE as i8,
            Some(MouseButton::Right) => PROCESSING_MOUSE_RIGHT as i8,
            _ => -1,
        })
    })
    .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_key_is_pressed() -> bool {
    error::clear_error();
<<<<<<< HEAD
    error::check(input_key_is_pressed).unwrap_or(false)
=======
    error::check(|| input_key_is_pressed()).unwrap_or(false)
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_key_is_down(key_code: u32) -> bool {
    error::clear_error();
    error::check(|| {
        let kc = key_code_from_u32(key_code)?;
        input_key_is_down(kc)
    })
    .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_key() -> u32 {
    error::clear_error();
    error::check(|| input_key().map(|opt| opt.map(|c| c as u32).unwrap_or(0))).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_key_code() -> u32 {
    error::clear_error();
<<<<<<< HEAD
<<<<<<< HEAD
    error::check(|| input_key_code().map(|opt| opt.map(key_code_to_u32).unwrap_or(0))).unwrap_or(0)
=======
    error::check(|| input_key_code().map(|opt| opt.map(key_code_to_u32).unwrap_or(0)))
        .unwrap_or(0)
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
=======
    error::check(|| input_key_code().map(|opt| opt.map(key_code_to_u32).unwrap_or(0))).unwrap_or(0)
>>>>>>> 2715c70 (Fmt.)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_moved_x() -> f32 {
    error::clear_error();
<<<<<<< HEAD
    error::check(input_moved_x).unwrap_or(0.0)
=======
    error::check(|| input_moved_x()).unwrap_or(0.0)
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_moved_y() -> f32 {
    error::clear_error();
<<<<<<< HEAD
    error::check(input_moved_y).unwrap_or(0.0)
=======
    error::check(|| input_moved_y()).unwrap_or(0.0)
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_mouse_wheel() -> f32 {
    error::clear_error();
<<<<<<< HEAD
    error::check(input_mouse_wheel).unwrap_or(0.0)
=======
    error::check(|| input_mouse_wheel()).unwrap_or(0.0)
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
}

fn key_code_from_u32(val: u32) -> processing::prelude::error::Result<KeyCode> {
    match val {
        PROCESSING_KEY_SPACE => Ok(KeyCode::Space),
        PROCESSING_KEY_QUOTE => Ok(KeyCode::Quote),
        PROCESSING_KEY_COMMA => Ok(KeyCode::Comma),
        PROCESSING_KEY_MINUS => Ok(KeyCode::Minus),
        PROCESSING_KEY_PERIOD => Ok(KeyCode::Period),
        PROCESSING_KEY_SLASH => Ok(KeyCode::Slash),
        PROCESSING_KEY_0 => Ok(KeyCode::Digit0),
        PROCESSING_KEY_1 => Ok(KeyCode::Digit1),
        PROCESSING_KEY_2 => Ok(KeyCode::Digit2),
        PROCESSING_KEY_3 => Ok(KeyCode::Digit3),
        PROCESSING_KEY_4 => Ok(KeyCode::Digit4),
        PROCESSING_KEY_5 => Ok(KeyCode::Digit5),
        PROCESSING_KEY_6 => Ok(KeyCode::Digit6),
        PROCESSING_KEY_7 => Ok(KeyCode::Digit7),
        PROCESSING_KEY_8 => Ok(KeyCode::Digit8),
        PROCESSING_KEY_9 => Ok(KeyCode::Digit9),
        PROCESSING_KEY_SEMICOLON => Ok(KeyCode::Semicolon),
        PROCESSING_KEY_EQUAL => Ok(KeyCode::Equal),
        PROCESSING_KEY_A => Ok(KeyCode::KeyA),
        PROCESSING_KEY_B => Ok(KeyCode::KeyB),
        PROCESSING_KEY_C => Ok(KeyCode::KeyC),
        PROCESSING_KEY_D => Ok(KeyCode::KeyD),
        PROCESSING_KEY_E => Ok(KeyCode::KeyE),
        PROCESSING_KEY_F => Ok(KeyCode::KeyF),
        PROCESSING_KEY_G => Ok(KeyCode::KeyG),
        PROCESSING_KEY_H => Ok(KeyCode::KeyH),
        PROCESSING_KEY_I => Ok(KeyCode::KeyI),
        PROCESSING_KEY_J => Ok(KeyCode::KeyJ),
        PROCESSING_KEY_K => Ok(KeyCode::KeyK),
        PROCESSING_KEY_L => Ok(KeyCode::KeyL),
        PROCESSING_KEY_M => Ok(KeyCode::KeyM),
        PROCESSING_KEY_N => Ok(KeyCode::KeyN),
        PROCESSING_KEY_O => Ok(KeyCode::KeyO),
        PROCESSING_KEY_P => Ok(KeyCode::KeyP),
        PROCESSING_KEY_Q => Ok(KeyCode::KeyQ),
        PROCESSING_KEY_R => Ok(KeyCode::KeyR),
        PROCESSING_KEY_S => Ok(KeyCode::KeyS),
        PROCESSING_KEY_T => Ok(KeyCode::KeyT),
        PROCESSING_KEY_U => Ok(KeyCode::KeyU),
        PROCESSING_KEY_V => Ok(KeyCode::KeyV),
        PROCESSING_KEY_W => Ok(KeyCode::KeyW),
        PROCESSING_KEY_X => Ok(KeyCode::KeyX),
        PROCESSING_KEY_Y => Ok(KeyCode::KeyY),
        PROCESSING_KEY_Z => Ok(KeyCode::KeyZ),
        PROCESSING_KEY_BRACKET_LEFT => Ok(KeyCode::BracketLeft),
        PROCESSING_KEY_BACKSLASH => Ok(KeyCode::Backslash),
        PROCESSING_KEY_BRACKET_RIGHT => Ok(KeyCode::BracketRight),
        PROCESSING_KEY_BACKQUOTE => Ok(KeyCode::Backquote),
        PROCESSING_KEY_ESCAPE => Ok(KeyCode::Escape),
        PROCESSING_KEY_ENTER => Ok(KeyCode::Enter),
        PROCESSING_KEY_TAB => Ok(KeyCode::Tab),
        PROCESSING_KEY_BACKSPACE => Ok(KeyCode::Backspace),
        PROCESSING_KEY_INSERT => Ok(KeyCode::Insert),
        PROCESSING_KEY_DELETE => Ok(KeyCode::Delete),
        PROCESSING_KEY_RIGHT => Ok(KeyCode::ArrowRight),
        PROCESSING_KEY_LEFT => Ok(KeyCode::ArrowLeft),
        PROCESSING_KEY_DOWN => Ok(KeyCode::ArrowDown),
        PROCESSING_KEY_UP => Ok(KeyCode::ArrowUp),
        PROCESSING_KEY_PAGE_UP => Ok(KeyCode::PageUp),
        PROCESSING_KEY_PAGE_DOWN => Ok(KeyCode::PageDown),
        PROCESSING_KEY_HOME => Ok(KeyCode::Home),
        PROCESSING_KEY_END => Ok(KeyCode::End),
        PROCESSING_KEY_CAPS_LOCK => Ok(KeyCode::CapsLock),
        PROCESSING_KEY_SCROLL_LOCK => Ok(KeyCode::ScrollLock),
        PROCESSING_KEY_NUM_LOCK => Ok(KeyCode::NumLock),
        PROCESSING_KEY_PRINT_SCREEN => Ok(KeyCode::PrintScreen),
        PROCESSING_KEY_PAUSE => Ok(KeyCode::Pause),
        PROCESSING_KEY_F1 => Ok(KeyCode::F1),
        PROCESSING_KEY_F2 => Ok(KeyCode::F2),
        PROCESSING_KEY_F3 => Ok(KeyCode::F3),
        PROCESSING_KEY_F4 => Ok(KeyCode::F4),
        PROCESSING_KEY_F5 => Ok(KeyCode::F5),
        PROCESSING_KEY_F6 => Ok(KeyCode::F6),
        PROCESSING_KEY_F7 => Ok(KeyCode::F7),
        PROCESSING_KEY_F8 => Ok(KeyCode::F8),
        PROCESSING_KEY_F9 => Ok(KeyCode::F9),
        PROCESSING_KEY_F10 => Ok(KeyCode::F10),
        PROCESSING_KEY_F11 => Ok(KeyCode::F11),
        PROCESSING_KEY_F12 => Ok(KeyCode::F12),
        PROCESSING_KEY_NUMPAD_0 => Ok(KeyCode::Numpad0),
        PROCESSING_KEY_NUMPAD_1 => Ok(KeyCode::Numpad1),
        PROCESSING_KEY_NUMPAD_2 => Ok(KeyCode::Numpad2),
        PROCESSING_KEY_NUMPAD_3 => Ok(KeyCode::Numpad3),
        PROCESSING_KEY_NUMPAD_4 => Ok(KeyCode::Numpad4),
        PROCESSING_KEY_NUMPAD_5 => Ok(KeyCode::Numpad5),
        PROCESSING_KEY_NUMPAD_6 => Ok(KeyCode::Numpad6),
        PROCESSING_KEY_NUMPAD_7 => Ok(KeyCode::Numpad7),
        PROCESSING_KEY_NUMPAD_8 => Ok(KeyCode::Numpad8),
        PROCESSING_KEY_NUMPAD_9 => Ok(KeyCode::Numpad9),
        PROCESSING_KEY_NUMPAD_DECIMAL => Ok(KeyCode::NumpadDecimal),
        PROCESSING_KEY_NUMPAD_DIVIDE => Ok(KeyCode::NumpadDivide),
        PROCESSING_KEY_NUMPAD_MULTIPLY => Ok(KeyCode::NumpadMultiply),
        PROCESSING_KEY_NUMPAD_SUBTRACT => Ok(KeyCode::NumpadSubtract),
        PROCESSING_KEY_NUMPAD_ADD => Ok(KeyCode::NumpadAdd),
        PROCESSING_KEY_NUMPAD_ENTER => Ok(KeyCode::NumpadEnter),
        PROCESSING_KEY_NUMPAD_EQUAL => Ok(KeyCode::NumpadEqual),
        PROCESSING_KEY_SHIFT_LEFT => Ok(KeyCode::ShiftLeft),
        PROCESSING_KEY_CONTROL_LEFT => Ok(KeyCode::ControlLeft),
        PROCESSING_KEY_ALT_LEFT => Ok(KeyCode::AltLeft),
        PROCESSING_KEY_SUPER_LEFT => Ok(KeyCode::SuperLeft),
        PROCESSING_KEY_SHIFT_RIGHT => Ok(KeyCode::ShiftRight),
        PROCESSING_KEY_CONTROL_RIGHT => Ok(KeyCode::ControlRight),
        PROCESSING_KEY_ALT_RIGHT => Ok(KeyCode::AltRight),
        PROCESSING_KEY_SUPER_RIGHT => Ok(KeyCode::SuperRight),
        PROCESSING_KEY_CONTEXT_MENU => Ok(KeyCode::ContextMenu),
<<<<<<< HEAD
<<<<<<< HEAD
        _ => Err(ProcessingError::InvalidArgument(format!(
            "unknown key code: {val}"
        ))),
=======
        _ => Err(ProcessingError::InvalidArgument(
            format!("unknown key code: {val}"),
        )),
>>>>>>> 6ad893c (Add input support; extract glfw crate.)
=======
        _ => Err(ProcessingError::InvalidArgument(format!(
            "unknown key code: {val}"
        ))),
>>>>>>> 2715c70 (Fmt.)
    }
}

fn key_code_to_u32(kc: KeyCode) -> u32 {
    match kc {
        KeyCode::Space => PROCESSING_KEY_SPACE,
        KeyCode::Quote => PROCESSING_KEY_QUOTE,
        KeyCode::Comma => PROCESSING_KEY_COMMA,
        KeyCode::Minus => PROCESSING_KEY_MINUS,
        KeyCode::Period => PROCESSING_KEY_PERIOD,
        KeyCode::Slash => PROCESSING_KEY_SLASH,
        KeyCode::Digit0 => PROCESSING_KEY_0,
        KeyCode::Digit1 => PROCESSING_KEY_1,
        KeyCode::Digit2 => PROCESSING_KEY_2,
        KeyCode::Digit3 => PROCESSING_KEY_3,
        KeyCode::Digit4 => PROCESSING_KEY_4,
        KeyCode::Digit5 => PROCESSING_KEY_5,
        KeyCode::Digit6 => PROCESSING_KEY_6,
        KeyCode::Digit7 => PROCESSING_KEY_7,
        KeyCode::Digit8 => PROCESSING_KEY_8,
        KeyCode::Digit9 => PROCESSING_KEY_9,
        KeyCode::Semicolon => PROCESSING_KEY_SEMICOLON,
        KeyCode::Equal => PROCESSING_KEY_EQUAL,
        KeyCode::KeyA => PROCESSING_KEY_A,
        KeyCode::KeyB => PROCESSING_KEY_B,
        KeyCode::KeyC => PROCESSING_KEY_C,
        KeyCode::KeyD => PROCESSING_KEY_D,
        KeyCode::KeyE => PROCESSING_KEY_E,
        KeyCode::KeyF => PROCESSING_KEY_F,
        KeyCode::KeyG => PROCESSING_KEY_G,
        KeyCode::KeyH => PROCESSING_KEY_H,
        KeyCode::KeyI => PROCESSING_KEY_I,
        KeyCode::KeyJ => PROCESSING_KEY_J,
        KeyCode::KeyK => PROCESSING_KEY_K,
        KeyCode::KeyL => PROCESSING_KEY_L,
        KeyCode::KeyM => PROCESSING_KEY_M,
        KeyCode::KeyN => PROCESSING_KEY_N,
        KeyCode::KeyO => PROCESSING_KEY_O,
        KeyCode::KeyP => PROCESSING_KEY_P,
        KeyCode::KeyQ => PROCESSING_KEY_Q,
        KeyCode::KeyR => PROCESSING_KEY_R,
        KeyCode::KeyS => PROCESSING_KEY_S,
        KeyCode::KeyT => PROCESSING_KEY_T,
        KeyCode::KeyU => PROCESSING_KEY_U,
        KeyCode::KeyV => PROCESSING_KEY_V,
        KeyCode::KeyW => PROCESSING_KEY_W,
        KeyCode::KeyX => PROCESSING_KEY_X,
        KeyCode::KeyY => PROCESSING_KEY_Y,
        KeyCode::KeyZ => PROCESSING_KEY_Z,
        KeyCode::BracketLeft => PROCESSING_KEY_BRACKET_LEFT,
        KeyCode::Backslash => PROCESSING_KEY_BACKSLASH,
        KeyCode::BracketRight => PROCESSING_KEY_BRACKET_RIGHT,
        KeyCode::Backquote => PROCESSING_KEY_BACKQUOTE,
        KeyCode::Escape => PROCESSING_KEY_ESCAPE,
        KeyCode::Enter => PROCESSING_KEY_ENTER,
        KeyCode::Tab => PROCESSING_KEY_TAB,
        KeyCode::Backspace => PROCESSING_KEY_BACKSPACE,
        KeyCode::Insert => PROCESSING_KEY_INSERT,
        KeyCode::Delete => PROCESSING_KEY_DELETE,
        KeyCode::ArrowRight => PROCESSING_KEY_RIGHT,
        KeyCode::ArrowLeft => PROCESSING_KEY_LEFT,
        KeyCode::ArrowDown => PROCESSING_KEY_DOWN,
        KeyCode::ArrowUp => PROCESSING_KEY_UP,
        KeyCode::PageUp => PROCESSING_KEY_PAGE_UP,
        KeyCode::PageDown => PROCESSING_KEY_PAGE_DOWN,
        KeyCode::Home => PROCESSING_KEY_HOME,
        KeyCode::End => PROCESSING_KEY_END,
        KeyCode::CapsLock => PROCESSING_KEY_CAPS_LOCK,
        KeyCode::ScrollLock => PROCESSING_KEY_SCROLL_LOCK,
        KeyCode::NumLock => PROCESSING_KEY_NUM_LOCK,
        KeyCode::PrintScreen => PROCESSING_KEY_PRINT_SCREEN,
        KeyCode::Pause => PROCESSING_KEY_PAUSE,
        KeyCode::F1 => PROCESSING_KEY_F1,
        KeyCode::F2 => PROCESSING_KEY_F2,
        KeyCode::F3 => PROCESSING_KEY_F3,
        KeyCode::F4 => PROCESSING_KEY_F4,
        KeyCode::F5 => PROCESSING_KEY_F5,
        KeyCode::F6 => PROCESSING_KEY_F6,
        KeyCode::F7 => PROCESSING_KEY_F7,
        KeyCode::F8 => PROCESSING_KEY_F8,
        KeyCode::F9 => PROCESSING_KEY_F9,
        KeyCode::F10 => PROCESSING_KEY_F10,
        KeyCode::F11 => PROCESSING_KEY_F11,
        KeyCode::F12 => PROCESSING_KEY_F12,
        KeyCode::Numpad0 => PROCESSING_KEY_NUMPAD_0,
        KeyCode::Numpad1 => PROCESSING_KEY_NUMPAD_1,
        KeyCode::Numpad2 => PROCESSING_KEY_NUMPAD_2,
        KeyCode::Numpad3 => PROCESSING_KEY_NUMPAD_3,
        KeyCode::Numpad4 => PROCESSING_KEY_NUMPAD_4,
        KeyCode::Numpad5 => PROCESSING_KEY_NUMPAD_5,
        KeyCode::Numpad6 => PROCESSING_KEY_NUMPAD_6,
        KeyCode::Numpad7 => PROCESSING_KEY_NUMPAD_7,
        KeyCode::Numpad8 => PROCESSING_KEY_NUMPAD_8,
        KeyCode::Numpad9 => PROCESSING_KEY_NUMPAD_9,
        KeyCode::NumpadDecimal => PROCESSING_KEY_NUMPAD_DECIMAL,
        KeyCode::NumpadDivide => PROCESSING_KEY_NUMPAD_DIVIDE,
        KeyCode::NumpadMultiply => PROCESSING_KEY_NUMPAD_MULTIPLY,
        KeyCode::NumpadSubtract => PROCESSING_KEY_NUMPAD_SUBTRACT,
        KeyCode::NumpadAdd => PROCESSING_KEY_NUMPAD_ADD,
        KeyCode::NumpadEnter => PROCESSING_KEY_NUMPAD_ENTER,
        KeyCode::NumpadEqual => PROCESSING_KEY_NUMPAD_EQUAL,
        KeyCode::ShiftLeft => PROCESSING_KEY_SHIFT_LEFT,
        KeyCode::ControlLeft => PROCESSING_KEY_CONTROL_LEFT,
        KeyCode::AltLeft => PROCESSING_KEY_ALT_LEFT,
        KeyCode::SuperLeft => PROCESSING_KEY_SUPER_LEFT,
        KeyCode::ShiftRight => PROCESSING_KEY_SHIFT_RIGHT,
        KeyCode::ControlRight => PROCESSING_KEY_CONTROL_RIGHT,
        KeyCode::AltRight => PROCESSING_KEY_ALT_RIGHT,
        KeyCode::SuperRight => PROCESSING_KEY_SUPER_RIGHT,
        KeyCode::ContextMenu => PROCESSING_KEY_CONTEXT_MENU,
        _ => 0,
    }
}
