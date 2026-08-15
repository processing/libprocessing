use bevy::{
    math::{Affine3A, Mat4, Vec2, Vec3, Vec4},
    prelude::Entity,
    render::render_resource::{Extent3d, TextureFormat},
};
use processing::prelude::{error::ProcessingError, shader_value::ShaderValue, *};

use crate::color::Color;

mod color;
mod error;

unsafe fn cstr_to_str<'a>(ptr: *const std::ffi::c_char) -> Result<&'a str, ProcessingError> {
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|_| ProcessingError::InvalidArgument("non-UTF8 C string".to_string()))
}

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

/// Initialize libProcessing with an asset root directory. Relative paths
/// passed to `processing_shader_load` (and image/gltf loads) resolve
/// against this directory. Same constraints as `processing_init`.
///
/// SAFETY:
/// - `asset_root` must be non-null.
/// - This is called from the main thread if the platform requires it.
/// - This can only be called once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_init_with_asset_root(asset_root: *const std::ffi::c_char) {
    error::clear_error();
    error::check(|| {
        let asset_root = unsafe { cstr_to_str(asset_root) }?;
        let mut config = Config::default();
        if !asset_root.is_empty() {
            config.set(ConfigKey::AssetRootPath, asset_root.to_string());
        }
        init(config)
    });
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
    error::check(|| surface_create_macos(window_handle, width, height, scale_factor, false))
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

/// Create a WebGPU surface on Linux. The display server is auto-detected from
/// the environment.
///
/// SAFETY:
/// - Init has been called.
/// - The handle types match the active display server.
/// - This is called from the same thread as init.
#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub extern "C" fn processing_surface_create(
    window_handle: u64,
    display_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> u64 {
    error::clear_error();
    error::check(|| {
        surface_create_linux(window_handle, display_handle, width, height, scale_factor)
    })
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

/// Clear the graphics surface to transparent.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_clear(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::BackgroundColor(bevy::prelude::Color::NONE),
        )
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

/// Set the rect mode.
#[unsafe(no_mangle)]
pub extern "C" fn processing_rect_mode(graphics_id: u64, mode: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::RectMode(processing::prelude::ShapeMode::from(mode)),
        )
    });
}

/// Set the ellipse mode.
#[unsafe(no_mangle)]
pub extern "C" fn processing_ellipse_mode(graphics_id: u64, mode: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::EllipseMode(processing::prelude::ShapeMode::from(mode)),
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

/// Push the current style onto the style stack.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_push_style(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::PushStyle));
}

/// Pop the most recently saved style off the style stack.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_pop_style(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::PopStyle));
}

/// Push both the style and the transformation matrix onto their stacks.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_push(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::PushStyle)?;
        graphics_record_command(graphics_entity, DrawCommand::PushMatrix)
    });
}

/// Pop both the style and the transformation matrix off their stacks.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_pop(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::PopStyle)?;
        graphics_record_command(graphics_entity, DrawCommand::PopMatrix)
    });
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
pub extern "C" fn processing_translate(graphics_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::Translate(Vec3::new(x, y, z)))
    });
}

/// Rotate the coordinate system by `angle` about the axis (x, y, z).
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_rotate(graphics_id: u64, angle: f32, x: f32, y: f32, z: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Rotate {
                angle,
                axis: Vec3::new(x, y, z),
            },
        )
    });
}

/// Scale the coordinate system.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_scale(graphics_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::Scale(Vec3::new(x, y, z)))
    });
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

#[unsafe(no_mangle)]
pub extern "C" fn processing_set_blend_mode(graphics_id: u64, mode: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        let blend_state = processing::prelude::BlendMode::try_from(mode)?.to_blend_state();
        graphics_record_command(graphics_entity, DrawCommand::BlendMode(blend_state))
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_set_custom_blend_mode(
    graphics_id: u64,
    color_src: u8,
    color_dst: u8,
    color_op: u8,
    alpha_src: u8,
    alpha_dst: u8,
    alpha_op: u8,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        let blend_state = custom_blend_state(
            color_src, color_dst, color_op, alpha_src, alpha_dst, alpha_op,
        )?;
        graphics_record_command(graphics_entity, DrawCommand::BlendMode(Some(blend_state)))
    });
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

/// Draw an ellipse.
#[unsafe(no_mangle)]
pub extern "C" fn processing_ellipse(graphics_id: u64, cx: f32, cy: f32, w: f32, h: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::Ellipse { cx, cy, w, h })
    });
}

/// Draw a circle.
#[unsafe(no_mangle)]
pub extern "C" fn processing_circle(graphics_id: u64, cx: f32, cy: f32, d: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::Ellipse { cx, cy, w: d, h: d })
    });
}

/// Draw a line.
#[unsafe(no_mangle)]
pub extern "C" fn processing_line(graphics_id: u64, x1: f32, y1: f32, x2: f32, y2: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Line { x1, y1, x2, y2 }));
}

/// Draw a triangle.
#[unsafe(no_mangle)]
pub extern "C" fn processing_triangle(
    graphics_id: u64,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Triangle {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            },
        )
    });
}

/// Draw a quadrilateral.
#[unsafe(no_mangle)]
pub extern "C" fn processing_quad(
    graphics_id: u64,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    x4: f32,
    y4: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Quad {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            },
        )
    });
}

/// Draw a point.
#[unsafe(no_mangle)]
pub extern "C" fn processing_point(graphics_id: u64, x: f32, y: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Point { x, y }));
}

/// Draw a square.
#[unsafe(no_mangle)]
pub extern "C" fn processing_square(graphics_id: u64, x: f32, y: f32, s: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Rect {
                x,
                y,
                w: s,
                h: s,
                radii: [0.0; 4],
            },
        )
    });
}

/// Draw an arc.
#[unsafe(no_mangle)]
pub extern "C" fn processing_arc(
    graphics_id: u64,
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    start: f32,
    stop: f32,
    mode: u8,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Arc {
                cx,
                cy,
                w,
                h,
                start,
                stop,
                mode: processing::prelude::ArcMode::from(mode),
            },
        )
    });
}

/// Draw a cubic bezier curve.
#[unsafe(no_mangle)]
pub extern "C" fn processing_bezier(
    graphics_id: u64,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    x4: f32,
    y4: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Bezier {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            },
        )
    });
}

/// Draw a Catmull-Rom curve.
#[unsafe(no_mangle)]
pub extern "C" fn processing_curve(
    graphics_id: u64,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    x4: f32,
    y4: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Curve {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            },
        )
    });
}

/// Draw a cylinder.
#[unsafe(no_mangle)]
pub extern "C" fn processing_cylinder(graphics_id: u64, radius: f32, height: f32, detail: u32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Cylinder {
                radius,
                height,
                detail,
            },
        )
    });
}

/// Draw a cone.
#[unsafe(no_mangle)]
pub extern "C" fn processing_cone(graphics_id: u64, radius: f32, height: f32, detail: u32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Cone {
                radius,
                height,
                detail,
            },
        )
    });
}

/// Draw a torus.
#[unsafe(no_mangle)]
pub extern "C" fn processing_torus(
    graphics_id: u64,
    radius: f32,
    tube_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Torus {
                radius,
                tube_radius,
                major_segments,
                minor_segments,
            },
        )
    });
}

/// Draw a plane.
#[unsafe(no_mangle)]
pub extern "C" fn processing_plane(graphics_id: u64, width: f32, height: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Plane { width, height }));
}

/// Draw a capsule.
#[unsafe(no_mangle)]
pub extern "C" fn processing_capsule(graphics_id: u64, radius: f32, length: f32, detail: u32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Capsule {
                radius,
                length,
                detail,
            },
        )
    });
}

/// Draw a conical frustum.
#[unsafe(no_mangle)]
pub extern "C" fn processing_conical_frustum(
    graphics_id: u64,
    radius_top: f32,
    radius_bottom: f32,
    height: f32,
    detail: u32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::ConicalFrustum {
                radius_top,
                radius_bottom,
                height,
                detail,
            },
        )
    });
}

/// Draw a tetrahedron.
#[unsafe(no_mangle)]
pub extern "C" fn processing_tetrahedron(graphics_id: u64, radius: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::Tetrahedron { radius }));
}

/// Begin recording vertices for a custom shape.
#[unsafe(no_mangle)]
pub extern "C" fn processing_begin_shape(graphics_id: u64, kind: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::BeginShape {
                kind: processing::prelude::ShapeKind::from(kind),
            },
        )
    });
}

/// End recording vertices and draw the shape.
#[unsafe(no_mangle)]
pub extern "C" fn processing_end_shape(graphics_id: u64, close: bool) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::EndShape { close }));
}

/// Add a vertex to the current shape.
#[unsafe(no_mangle)]
pub extern "C" fn processing_vertex(graphics_id: u64, x: f32, y: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::ShapeVertex { x, y }));
}

/// Add a cubic bezier vertex to the current shape.
#[unsafe(no_mangle)]
pub extern "C" fn processing_bezier_vertex(
    graphics_id: u64,
    cx1: f32,
    cy1: f32,
    cx2: f32,
    cy2: f32,
    x: f32,
    y: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::ShapeBezierVertex {
                cx1,
                cy1,
                cx2,
                cy2,
                x,
                y,
            },
        )
    });
}

/// Add a quadratic bezier vertex to the current shape.
#[unsafe(no_mangle)]
pub extern "C" fn processing_quadratic_vertex(graphics_id: u64, cx: f32, cy: f32, x: f32, y: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::ShapeQuadraticVertex { cx, cy, x, y },
        )
    });
}

/// Add a Catmull-Rom curve vertex to the current shape.
#[unsafe(no_mangle)]
pub extern "C" fn processing_curve_vertex(graphics_id: u64, x: f32, y: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(graphics_entity, DrawCommand::ShapeCurveVertex { x, y })
    });
}

/// Begin a contour within the current shape.
#[unsafe(no_mangle)]
pub extern "C" fn processing_begin_contour(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::BeginContour));
}

/// End the current contour.
#[unsafe(no_mangle)]
pub extern "C" fn processing_end_contour(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::EndContour));
}

// --- Font ---

/// Load a font file and return a font entity ID.
/// Returns 0 on error.
///
/// # Safety
/// - path_ptr is a valid pointer to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_load_font(path_ptr: *const std::ffi::c_char) -> u64 {
    error::clear_error();
    let path = unsafe { std::ffi::CStr::from_ptr(path_ptr) }.to_string_lossy();
    error::check(|| font_load(&path).map(|e| e.to_bits())).unwrap_or(0)
}

/// Create a font handle from an existing font family name.
/// Returns 0 on error.
///
/// # Safety
/// - name_ptr is a valid pointer to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_create_font(name_ptr: *const std::ffi::c_char) -> u64 {
    error::clear_error();
    let name = unsafe { std::ffi::CStr::from_ptr(name_ptr) }.to_string_lossy();
    error::check(|| font_create(&name).map(|e| e.to_bits())).unwrap_or(0)
}

/// Query the number of variable font axes for a font.
/// Returns 0 if the font is not variable or not found.
#[unsafe(no_mangle)]
pub extern "C" fn processing_font_variation_count(font_id: u64) -> u32 {
    error::clear_error();
    let font_entity = Entity::from_bits(font_id);
    error::check(|| font_variations(font_entity).map(|v| v.len() as u32)).unwrap_or(0)
}

/// Query variable font axis info.
/// Writes tag (4 bytes), min, max, default to out buffer at the given index.
///
/// # Safety
/// - out_tag is a valid pointer to at least 4 writable bytes.
/// - out_min, out_max, out_default are valid pointers to writable f32 values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_font_variation(
    font_id: u64,
    index: u32,
    out_tag: *mut u8,
    out_min: *mut f32,
    out_max: *mut f32,
    out_default: *mut f32,
) -> bool {
    error::clear_error();
    let font_entity = Entity::from_bits(font_id);
    let axes = error::check(|| font_variations(font_entity));
    if let Some(axes) = axes
        && let Some(axis) = axes.get(index as usize)
    {
        let tag_bytes = axis.tag.as_bytes();
        let len = tag_bytes.len().min(4);
        unsafe {
            std::ptr::copy_nonoverlapping(tag_bytes.as_ptr(), out_tag, len);
            for i in len..4 {
                *out_tag.add(i) = b' ';
            }
            *out_min = axis.min;
            *out_max = axis.max;
            *out_default = axis.default;
        }
        return true;
    }
    false
}

/// Set the current text font.
/// Pass 0 to reset to the default font.
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_font(graphics_id: u64, font_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let font_entity = if font_id == 0 {
        None
    } else {
        Some(Entity::from_bits(font_id))
    };
    error::check(|| graphics_text_font(graphics_entity, font_entity));
}

// --- Text ---

/// Draw text at a position.
///
/// # Safety
/// - str_ptr is a valid pointer to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_text(
    graphics_id: u64,
    str_ptr: *const std::ffi::c_char,
    x: f32,
    y: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = unsafe { std::ffi::CStr::from_ptr(str_ptr) }
        .to_string_lossy()
        .into_owned();
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Text {
                content,
                x,
                y,
                z: 0.0,
                max_w: None,
                max_h: None,
            },
        )
    });
}

/// Draw text at a 3D position.
///
/// # Safety
/// - str_ptr is a valid pointer to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_text_3d(
    graphics_id: u64,
    str_ptr: *const std::ffi::c_char,
    x: f32,
    y: f32,
    z: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = unsafe { std::ffi::CStr::from_ptr(str_ptr) }
        .to_string_lossy()
        .into_owned();
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Text {
                content,
                x,
                y,
                z,
                max_w: None,
                max_h: None,
            },
        )
    });
}

/// Draw an integer as text at a position.
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_int(graphics_id: u64, value: i32, x: f32, y: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = value.to_string();
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Text {
                content,
                x,
                y,
                z: 0.0,
                max_w: None,
                max_h: None,
            },
        )
    });
}

/// Draw a float as text at a position (formatted to 3 decimal places).
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_float(graphics_id: u64, value: f32, x: f32, y: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = format!("{:.3}", value);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Text {
                content,
                x,
                y,
                z: 0.0,
                max_w: None,
                max_h: None,
            },
        )
    });
}

/// Draw text within a bounding box (with word wrapping).
///
/// # Safety
/// - str_ptr is a valid pointer to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_text_box(
    graphics_id: u64,
    str_ptr: *const std::ffi::c_char,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = unsafe { std::ffi::CStr::from_ptr(str_ptr) }
        .to_string_lossy()
        .into_owned();
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Text {
                content,
                x,
                y,
                z: 0.0,
                max_w: Some(w),
                max_h: Some(h),
            },
        )
    });
}

/// Set the text style. 0=NORMAL, 1=ITALIC, 2=BOLD, 3=BOLDITALIC
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_style(graphics_id: u64, style: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_text_style(graphics_entity, style));
}

/// Compute the bounding box of text. Writes [x, y, w, h] to out_bounds.
///
/// # Safety
/// - str_ptr is a valid pointer to a null-terminated string.
/// - out_bounds is a valid pointer to a writable float array of at least 4 elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_text_bounds(
    graphics_id: u64,
    str_ptr: *const std::ffi::c_char,
    x: f32,
    y: f32,
    out_bounds: *mut f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = unsafe { std::ffi::CStr::from_ptr(str_ptr) }.to_string_lossy();
    if let Some(bounds) =
        error::check(|| graphics_text_bounds(graphics_entity, &content, x, y, None, None))
    {
        unsafe {
            *out_bounds = bounds[0];
            *out_bounds.add(1) = bounds[1];
            *out_bounds.add(2) = bounds[2];
            *out_bounds.add(3) = bounds[3];
        }
    }
}

/// Set a font variation axis value (e.g. "wdth", 75.0).
///
/// # Safety
/// - tag_ptr is a valid pointer to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_text_variation(
    graphics_id: u64,
    tag_ptr: *const std::ffi::c_char,
    value: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let tag = unsafe { std::ffi::CStr::from_ptr(tag_ptr) }.to_string_lossy();
    error::check(|| graphics_text_variation(graphics_entity, &tag, value));
}

/// Clear all font variation axis overrides.
#[unsafe(no_mangle)]
pub extern "C" fn processing_clear_text_variations(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_clear_text_variations(graphics_entity));
}

/// Enable/configure an OpenType font feature (e.g. "smcp", 1).
///
/// # Safety
/// - tag_ptr is a valid pointer to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_text_feature(
    graphics_id: u64,
    tag_ptr: *const std::ffi::c_char,
    value: u16,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let tag = unsafe { std::ffi::CStr::from_ptr(tag_ptr) }.to_string_lossy();
    error::check(|| graphics_text_feature(graphics_entity, &tag, value));
}

/// Disable an OpenType font feature.
///
/// # Safety
/// - tag_ptr is a valid pointer to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_no_text_feature(
    graphics_id: u64,
    tag_ptr: *const std::ffi::c_char,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let tag = unsafe { std::ffi::CStr::from_ptr(tag_ptr) }.to_string_lossy();
    error::check(|| graphics_no_text_feature(graphics_entity, &tag));
}

/// Clear all OpenType font feature overrides.
#[unsafe(no_mangle)]
pub extern "C" fn processing_clear_text_features(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_clear_text_features(graphics_entity));
}

/// Set per-glyph colors for the next text() call.
/// colors_ptr points to an array of (r, g, b, a) float tuples.
///
/// # Safety
/// - colors_ptr is a valid pointer to count * 4 readable floats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_text_glyph_colors(
    graphics_id: u64,
    colors_ptr: *const f32,
    count: u32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let colors: Vec<bevy::color::Color> = (0..count as usize)
        .map(|i| unsafe {
            let base = colors_ptr.add(i * 4);
            bevy::color::Color::srgba(*base, *base.add(1), *base.add(2), *base.add(3))
        })
        .collect();
    error::check(|| graphics_text_glyph_colors(graphics_entity, colors));
}

/// Set the font weight for variable fonts (e.g. 100-900).
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_weight(graphics_id: u64, weight: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_text_weight(graphics_entity, weight));
}

/// Set the text size.
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_size(graphics_id: u64, size: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::TextSize(size)));
}

/// Set the text alignment.
/// h: 0=LEFT, 1=CENTER, 2=RIGHT
/// v: 0=BASELINE, 1=TOP, 2=CENTER, 3=BOTTOM
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_align(graphics_id: u64, h: u8, v: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_text_align(graphics_entity, h, v));
}

/// Set the text leading (line spacing).
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_leading(graphics_id: u64, leading: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::TextLeading(leading)));
}

/// Set the text wrap mode. 0=WORD, 1=CHAR
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_wrap(graphics_id: u64, mode: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_text_wrap(graphics_entity, mode));
}

/// Measure the width of text.
///
/// # Safety
/// - str_ptr is a valid pointer to a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_text_width(
    graphics_id: u64,
    str_ptr: *const std::ffi::c_char,
) -> f32 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = unsafe { std::ffi::CStr::from_ptr(str_ptr) }.to_string_lossy();
    error::check(|| graphics_text_width(graphics_entity, &content)).unwrap_or(0.0)
}

/// Get the text ascent for the current font size.
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_ascent(graphics_id: u64) -> f32 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_text_ascent(graphics_entity)).unwrap_or(0.0)
}

/// Get the text descent for the current font size.
#[unsafe(no_mangle)]
pub extern "C" fn processing_text_descent(graphics_id: u64) -> f32 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_text_descent(graphics_entity)).unwrap_or(0.0)
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

/// # Safety
/// - `init` has been called.
/// - `floats` is valid for `floats_len` f32 reads.
/// - Called from the same thread as `init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_image_create_hdr(
    width: u32,
    height: u32,
    floats: *const f32,
    floats_len: usize,
) -> u64 {
    error::clear_error();
    let src = unsafe { std::slice::from_raw_parts(floats, floats_len) };
    error::check(|| {
        let mut packed = Vec::with_capacity(src.len() * 2);
        for &f in src {
            packed.extend_from_slice(&half::f16::from_f32(f).to_le_bytes());
        }
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        image_create(size, packed, TextureFormat::Rgba16Float)
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

/// Load pixels from the graphics surface into a caller-provided buffer.
///
/// # Safety
/// - Init and graphics_create have been called.
/// - graphics_id is a valid ID returned from graphics_create.
/// - buffer is a valid pointer to at least buffer_len Color elements.
/// - buffer_len must equal width * height of the graphics surface.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_graphics_readback(
    graphics_id: u64,
    buffer: *mut Color,
    buffer_len: usize,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        let colors = graphics_readback(graphics_entity)?;

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

/// Write a caller-provided pixel buffer back onto the graphics surface.
///
/// # Safety
/// - Init and graphics_create have been called.
/// - graphics_id is a valid ID returned from graphics_create.
/// - buffer is a valid pointer to at least buffer_len Color elements.
/// - buffer_len must equal width * height of the graphics surface.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_graphics_update(
    graphics_id: u64,
    buffer: *const Color,
    buffer_len: usize,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        // SAFETY: Caller guarantees buffer is valid for buffer_len elements
        let pixels: Vec<_> = unsafe { std::slice::from_raw_parts(buffer, buffer_len) }
            .iter()
            .map(|color| color.to_linear())
            .collect();
        graphics_update(graphics_entity, &pixels)
    });
}

/// Write a caller-provided pixel buffer onto a rectangular region of the surface.
///
/// # Safety
/// - Init and graphics_create have been called.
/// - graphics_id is a valid ID returned from graphics_create.
/// - buffer is a valid pointer to at least buffer_len Color elements.
/// - buffer_len must equal width * height.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_graphics_update_region(
    graphics_id: u64,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    buffer: *const Color,
    buffer_len: usize,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        // SAFETY: Caller guarantees buffer is valid for buffer_len elements
        let pixels: Vec<_> = unsafe { std::slice::from_raw_parts(buffer, buffer_len) }
            .iter()
            .map(|color| color.to_linear())
            .collect();
        graphics_update_region(graphics_entity, x, y, width, height, &pixels)
    });
}

/// Set a single pixel on the graphics surface.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_graphics_set(graphics_id: u64, x: u32, y: u32, color: Color) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_update_region(graphics_entity, x, y, 1, 1, &[color.to_linear()]));
}

/// Set the tint color applied to images.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_tint(graphics_id: u64, color: Color) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        graphics_record_command(graphics_entity, DrawCommand::Tint(color.resolve(&mode)))
    });
}

/// Remove the image tint.
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_no_tint(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_record_command(graphics_entity, DrawCommand::NoTint));
}

/// Set how image() interprets its coordinates (CORNER/CORNERS/CENTER/RADIUS).
///
/// SAFETY:
/// - graphics_id is a valid ID returned from graphics_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_image_mode(graphics_id: u64, mode: u8) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::ImageMode(processing::prelude::ShapeMode::from(mode)),
        )
    });
}

/// Draw an image at (dx, dy) at its native size.
///
/// SAFETY:
/// - graphics_id and image_id are valid IDs from graphics_create/image_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_image(graphics_id: u64, image_id: u64, dx: f32, dy: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let image_entity = Entity::from_bits(image_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Image {
                entity: image_entity,
                dx,
                dy,
                d_width: None,
                d_height: None,
                sx: None,
                sy: None,
                s_width: None,
                s_height: None,
            },
        )
    });
}

/// Draw an image at (dx, dy) scaled to (d_width, d_height).
///
/// SAFETY:
/// - graphics_id and image_id are valid IDs from graphics_create/image_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_image_scaled(
    graphics_id: u64,
    image_id: u64,
    dx: f32,
    dy: f32,
    d_width: f32,
    d_height: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let image_entity = Entity::from_bits(image_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Image {
                entity: image_entity,
                dx,
                dy,
                d_width: Some(d_width),
                d_height: Some(d_height),
                sx: None,
                sy: None,
                s_width: None,
                s_height: None,
            },
        )
    });
}

/// Draw the (sx, sy, s_width, s_height) source region of an image into the
/// (dx, dy, d_width, d_height) destination rectangle.
///
/// SAFETY:
/// - graphics_id and image_id are valid IDs from graphics_create/image_create.
/// - This is called from the same thread as init.
#[unsafe(no_mangle)]
pub extern "C" fn processing_image_region(
    graphics_id: u64,
    image_id: u64,
    dx: f32,
    dy: f32,
    d_width: f32,
    d_height: f32,
    sx: f32,
    sy: f32,
    s_width: f32,
    s_height: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let image_entity = Entity::from_bits(image_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Image {
                entity: image_entity,
                dx,
                dy,
                d_width: Some(d_width),
                d_height: Some(d_height),
                sx: Some(sx),
                sy: Some(sy),
                s_width: Some(s_width),
                s_height: Some(s_height),
            },
        )
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

/// Attach an orbit camera controller.
#[unsafe(no_mangle)]
pub extern "C" fn processing_orbit_camera(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_orbit_camera(graphics_entity));
}

/// Attach a free-flight camera controller.
#[unsafe(no_mangle)]
pub extern "C" fn processing_free_camera(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_free_camera(graphics_entity));
}

/// Attach a pan/zoom camera controller.
#[unsafe(no_mangle)]
pub extern "C" fn processing_pan_camera(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_pan_camera(graphics_entity));
}

/// Remove the active camera controller.
#[unsafe(no_mangle)]
pub extern "C" fn processing_disable_camera_controller(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_disable_camera_controller(graphics_entity));
}

/// Set the camera distance from its center (zoom).
#[unsafe(no_mangle)]
pub extern "C" fn processing_camera_set_distance(graphics_id: u64, distance: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| camera_set_distance(graphics_entity, distance));
}

/// Set the orbit camera's look-at center.
#[unsafe(no_mangle)]
pub extern "C" fn processing_camera_set_center(graphics_id: u64, x: f32, y: f32, z: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| camera_set_center(graphics_entity, Vec3::new(x, y, z)));
}

/// Set the minimum camera distance.
#[unsafe(no_mangle)]
pub extern "C" fn processing_camera_set_min_distance(graphics_id: u64, min: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| camera_set_min_distance(graphics_entity, min));
}

/// Set the maximum camera distance.
#[unsafe(no_mangle)]
pub extern "C" fn processing_camera_set_max_distance(graphics_id: u64, max: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| camera_set_max_distance(graphics_entity, max));
}

/// Set the camera controller's sensitivity.
#[unsafe(no_mangle)]
pub extern "C" fn processing_camera_set_speed(graphics_id: u64, speed: f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| camera_set_speed(graphics_entity, speed));
}

/// Reset the camera controller to its initial pose.
#[unsafe(no_mangle)]
pub extern "C" fn processing_camera_reset(graphics_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| camera_reset(graphics_entity));
}

/// Position the camera at eye, looking at center with the given up.
///
/// An active camera controller overrides this each frame.
#[unsafe(no_mangle)]
pub extern "C" fn processing_camera(
    graphics_id: u64,
    eye_x: f32,
    eye_y: f32,
    eye_z: f32,
    center_x: f32,
    center_y: f32,
    center_z: f32,
    up_x: f32,
    up_y: f32,
    up_z: f32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_camera(
            graphics_entity,
            Vec3::new(eye_x, eye_y, eye_z),
            Vec3::new(center_x, center_y, center_z),
            Vec3::new(up_x, up_y, up_z),
        )
    });
}

/// A column-major 4x4 matrix.
#[repr(C)]
pub struct Matrix {
    pub m: [f32; 16],
}

/// Right-multiply the model matrix by a column-major 4x4 matrix.
///
/// # Safety
/// - matrix points to at least 16 f32.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_apply_matrix(graphics_id: u64, matrix: *const f32) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        // SAFETY: Caller guarantees matrix points to 16 valid f32 elements
        let cols: [f32; 16] = unsafe { std::slice::from_raw_parts(matrix, 16) }
            .try_into()
            .unwrap();
        let affine = Affine3A::from_mat4(Mat4::from_cols_array(&cols));
        graphics_record_command(graphics_entity, DrawCommand::ApplyMatrix(affine))
    });
}

/// The current model matrix, column-major. Flushes pending draws; identity on error.
#[unsafe(no_mangle)]
pub extern "C" fn processing_get_matrix(graphics_id: u64) -> Matrix {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    let m = error::check(|| graphics_get_matrix(graphics_entity).map(|mat| mat.to_cols_array()))
        .unwrap_or_else(|| Mat4::IDENTITY.to_cols_array());
    Matrix { m }
}

/// Model-space point to world-space X (modelX).
#[unsafe(no_mangle)]
pub extern "C" fn processing_model_x(graphics_id: u64, x: f32, y: f32, z: f32) -> f32 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_model_point(graphics_entity, Vec3::new(x, y, z)).map(|p| p.x))
        .unwrap_or(0.0)
}

/// Model-space point to world-space Y (modelY).
#[unsafe(no_mangle)]
pub extern "C" fn processing_model_y(graphics_id: u64, x: f32, y: f32, z: f32) -> f32 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_model_point(graphics_entity, Vec3::new(x, y, z)).map(|p| p.y))
        .unwrap_or(0.0)
}

/// Model-space point to world-space Z (modelZ).
#[unsafe(no_mangle)]
pub extern "C" fn processing_model_z(graphics_id: u64, x: f32, y: f32, z: f32) -> f32 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_model_point(graphics_entity, Vec3::new(x, y, z)).map(|p| p.z))
        .unwrap_or(0.0)
}

/// Model-space point to screen X in pixels (screenX).
#[unsafe(no_mangle)]
pub extern "C" fn processing_screen_x(graphics_id: u64, x: f32, y: f32, z: f32) -> f32 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_screen_point(graphics_entity, Vec3::new(x, y, z)).map(|p| p.x))
        .unwrap_or(0.0)
}

/// Model-space point to screen Y in pixels (screenY).
#[unsafe(no_mangle)]
pub extern "C" fn processing_screen_y(graphics_id: u64, x: f32, y: f32, z: f32) -> f32 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_screen_point(graphics_entity, Vec3::new(x, y, z)).map(|p| p.y))
        .unwrap_or(0.0)
}

/// Model-space point to screen depth in [0,1] (screenZ).
#[unsafe(no_mangle)]
pub extern "C" fn processing_screen_z(graphics_id: u64, x: f32, y: f32, z: f32) -> f32 {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| graphics_screen_point(graphics_entity, Vec3::new(x, y, z)).map(|p| p.z))
        .unwrap_or(0.0)
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

pub const PROCESSING_BLEND_MODE_BLEND: u8 = 0;
pub const PROCESSING_BLEND_MODE_ADD: u8 = 1;
pub const PROCESSING_BLEND_MODE_SUBTRACT: u8 = 2;
pub const PROCESSING_BLEND_MODE_DARKEST: u8 = 3;
pub const PROCESSING_BLEND_MODE_LIGHTEST: u8 = 4;
pub const PROCESSING_BLEND_MODE_DIFFERENCE: u8 = 5;
pub const PROCESSING_BLEND_MODE_EXCLUSION: u8 = 6;
pub const PROCESSING_BLEND_MODE_MULTIPLY: u8 = 7;
pub const PROCESSING_BLEND_MODE_SCREEN: u8 = 8;
pub const PROCESSING_BLEND_MODE_REPLACE: u8 = 9;

pub const PROCESSING_BLEND_FACTOR_ZERO: u8 = 0;
pub const PROCESSING_BLEND_FACTOR_ONE: u8 = 1;
pub const PROCESSING_BLEND_FACTOR_SRC: u8 = 2;
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_SRC: u8 = 3;
pub const PROCESSING_BLEND_FACTOR_SRC_ALPHA: u8 = 4;
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA: u8 = 5;
pub const PROCESSING_BLEND_FACTOR_DST: u8 = 6;
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_DST: u8 = 7;
pub const PROCESSING_BLEND_FACTOR_DST_ALPHA: u8 = 8;
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_DST_ALPHA: u8 = 9;
pub const PROCESSING_BLEND_FACTOR_SRC_ALPHA_SATURATED: u8 = 10;

pub const PROCESSING_BLEND_OP_ADD: u8 = 0;
pub const PROCESSING_BLEND_OP_SUBTRACT: u8 = 1;
pub const PROCESSING_BLEND_OP_REVERSE_SUBTRACT: u8 = 2;
pub const PROCESSING_BLEND_OP_MIN: u8 = 3;
pub const PROCESSING_BLEND_OP_MAX: u8 = 4;

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
pub extern "C" fn processing_geometry_attribute_rotation() -> u64 {
    geometry_attribute_rotation().to_bits()
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_scale() -> u64 {
    geometry_attribute_scale().to_bits()
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_life() -> u64 {
    geometry_attribute_life().to_bits()
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_velocity() -> u64 {
    geometry_attribute_velocity().to_bits()
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_age() -> u64 {
    geometry_attribute_age().to_bits()
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_geometry_attribute_format(attr_id: u64) -> u8 {
    error::clear_error();
    error::check(|| {
        let (_name, fmt) = geometry_attribute_info(Entity::from_bits(attr_id))?;
        Ok(match fmt {
            geometry::AttributeFormat::Float => 1,
            geometry::AttributeFormat::Float2 => 2,
            geometry::AttributeFormat::Float3 => 3,
            geometry::AttributeFormat::Float4 => 4,
        })
    })
    .unwrap_or(0)
}

/// # Safety
/// - `out` is valid for `out_cap` byte writes (may be null when `out_cap == 0`
///   for a length query).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_geometry_attribute_name(
    attr_id: u64,
    out: *mut u8,
    out_cap: u64,
) -> u64 {
    error::clear_error();
    let Some((name, _)) = error::check(|| geometry_attribute_info(Entity::from_bits(attr_id)))
    else {
        return 0;
    };
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len();
    if out_cap > 0 && !out.is_null() {
        let copy_len = name_len.min((out_cap - 1) as usize);
        unsafe { std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), out, copy_len) };
        unsafe { *out.add(copy_len) = 0 };
    }
    name_len as u64
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

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_create_custom(shader_id: u64) -> u64 {
    error::clear_error();
    error::check(|| material_create_custom(Entity::from_bits(shader_id)))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// Set a float field on a material.
///
/// # Safety
/// - `name` is a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_material_set_float(
    mat_id: u64,
    name: *const std::ffi::c_char,
    value: f32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        material_set(
            Entity::from_bits(mat_id),
            name,
            shader_value::ShaderValue::Float(value),
        )
    });
}

/// Set a float4 field on a material.
///
/// # Safety
/// - `name` is a valid null-terminated C string.
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
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        material_set(
            Entity::from_bits(mat_id),
            name,
            shader_value::ShaderValue::Float4([r, g, b, a]),
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

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_set_alpha_mode(mat_id: u64, mode: u8, cutoff: f32) {
    error::clear_error();
    error::check(|| material_set_alpha_mode(Entity::from_bits(mat_id), mode, cutoff));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_set_double_sided(mat_id: u64, value: bool) {
    error::clear_error();
    error::check(|| material_set_double_sided(Entity::from_bits(mat_id), value));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_set_unlit(mat_id: u64, value: bool) {
    error::clear_error();
    error::check(|| material_set_unlit(Entity::from_bits(mat_id), value));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_set_depth_write(mat_id: u64, value: bool) {
    error::clear_error();
    error::check(|| material_set_depth_write(Entity::from_bits(mat_id), value));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_set_custom_blend_mode(
    mat_id: u64,
    color_src: u8,
    color_dst: u8,
    color_op: u8,
    alpha_src: u8,
    alpha_dst: u8,
    alpha_op: u8,
) {
    error::clear_error();
    error::check(|| {
        let blend_state = custom_blend_state(
            color_src, color_dst, color_op, alpha_src, alpha_dst, alpha_op,
        )?;
        material_set_custom_blend(Entity::from_bits(mat_id), blend_state)
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_graphics_apply_filter(graphics_id: u64, filter_id: u64) {
    error::clear_error();
    error::check(|| {
        graphics_apply_filter(Entity::from_bits(graphics_id), Entity::from_bits(filter_id))
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_create(shader_id: u64) -> u64 {
    error::clear_error();
    error::check(|| filter_create(Entity::from_bits(shader_id)))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_destroy(filter_id: u64) {
    error::clear_error();
    error::check(|| filter_destroy(Entity::from_bits(filter_id)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_set_passes(filter_id: u64, passes: u32) {
    error::clear_error();
    error::check(|| filter_set_passes(Entity::from_bits(filter_id), passes));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_blur() -> u64 {
    error::clear_error();
    error::check(|| filter_blur())
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_invert() -> u64 {
    error::clear_error();
    error::check(|| filter_invert())
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_gray() -> u64 {
    error::clear_error();
    error::check(|| filter_gray())
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_threshold() -> u64 {
    error::clear_error();
    error::check(|| filter_threshold())
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_posterize() -> u64 {
    error::clear_error();
    error::check(|| filter_posterize())
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_opaque() -> u64 {
    error::clear_error();
    error::check(|| filter_opaque())
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_erode() -> u64 {
    error::clear_error();
    error::check(|| filter_erode())
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_filter_dilate() -> u64 {
    error::clear_error();
    error::check(|| filter_dilate())
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// Create a shader from WGSL source.
///
/// # Safety
/// - `source` is a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_create(source: *const std::ffi::c_char) -> u64 {
    error::clear_error();
    error::check(|| {
        let source = unsafe { cstr_to_str(source) }?;
        shader_create(source)
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

/// # Safety
/// - `path` is a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_load(path: *const std::ffi::c_char) -> u64 {
    error::clear_error();
    error::check(|| {
        let path = unsafe { cstr_to_str(path) }?;
        shader_load(path)
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_shader_destroy(shader_id: u64) {
    error::clear_error();
    error::check(|| shader_destroy(Entity::from_bits(shader_id)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_buffer_create(size: u64) -> u64 {
    error::clear_error();
    error::check(|| buffer_create(size))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// # Safety
/// - `data` is valid for `len` byte reads.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_buffer_create_with_data(data: *const u8, len: u64) -> u64 {
    error::clear_error();
    let bytes = unsafe { std::slice::from_raw_parts(data, len as usize) }.to_vec();
    error::check(|| buffer_create_with_data(bytes))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// # Safety
/// - `data` is valid for `len` byte reads.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_buffer_write(buf_id: u64, data: *const u8, len: u64) {
    error::clear_error();
    let bytes = unsafe { std::slice::from_raw_parts(data, len as usize) }.to_vec();
    error::check(|| buffer_write(Entity::from_bits(buf_id), bytes));
}

/// Returns the byte length of a buffer, or 0 if not found (error is set).
#[unsafe(no_mangle)]
pub extern "C" fn processing_buffer_size(buf_id: u64) -> u64 {
    error::clear_error();
    error::check(|| buffer_size(Entity::from_bits(buf_id))).unwrap_or(0)
}

/// Read buffer contents into `out`. Returns the buffer's byte length;
/// `out` is only written when it fits in `out_len`. Pass `out_len == 0` for a
/// size query.
///
/// # Safety
/// - `out` is valid for `out_len` byte writes (may be null when `out_len == 0`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_buffer_read(buf_id: u64, out: *mut u8, out_len: u64) -> u64 {
    error::clear_error();
    let Some(data) = error::check(|| buffer_read(Entity::from_bits(buf_id))) else {
        return 0;
    };
    let needed = data.len() as u64;
    if needed <= out_len {
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), out, data.len()) };
    }
    needed
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_buffer_destroy(buf_id: u64) {
    error::clear_error();
    error::check(|| buffer_destroy(Entity::from_bits(buf_id)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_compute_create(shader_id: u64) -> u64 {
    error::clear_error();
    error::check(|| compute_create(Entity::from_bits(shader_id)))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_float(
    entity: u64,
    name: *const std::ffi::c_char,
    value: f32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(Entity::from_bits(entity), name, ShaderValue::Float(value))
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_int(
    entity: u64,
    name: *const std::ffi::c_char,
    value: i32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(Entity::from_bits(entity), name, ShaderValue::Int(value))
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_uint(
    entity: u64,
    name: *const std::ffi::c_char,
    value: u32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(Entity::from_bits(entity), name, ShaderValue::UInt(value))
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_vec2(
    entity: u64,
    name: *const std::ffi::c_char,
    x: f32,
    y: f32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(Entity::from_bits(entity), name, ShaderValue::Float2([x, y]))
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_vec3(
    entity: u64,
    name: *const std::ffi::c_char,
    x: f32,
    y: f32,
    z: f32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(
            Entity::from_bits(entity),
            name,
            ShaderValue::Float3([x, y, z]),
        )
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_vec4(
    entity: u64,
    name: *const std::ffi::c_char,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(
            Entity::from_bits(entity),
            name,
            ShaderValue::Float4([x, y, z, w]),
        )
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_ivec2(
    entity: u64,
    name: *const std::ffi::c_char,
    x: i32,
    y: i32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(Entity::from_bits(entity), name, ShaderValue::Int2([x, y]))
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_ivec3(
    entity: u64,
    name: *const std::ffi::c_char,
    x: i32,
    y: i32,
    z: i32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(
            Entity::from_bits(entity),
            name,
            ShaderValue::Int3([x, y, z]),
        )
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_ivec4(
    entity: u64,
    name: *const std::ffi::c_char,
    x: i32,
    y: i32,
    z: i32,
    w: i32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(
            Entity::from_bits(entity),
            name,
            ShaderValue::Int4([x, y, z, w]),
        )
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_uvec2(
    entity: u64,
    name: *const std::ffi::c_char,
    x: u32,
    y: u32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(Entity::from_bits(entity), name, ShaderValue::UInt2([x, y]))
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_uvec3(
    entity: u64,
    name: *const std::ffi::c_char,
    x: u32,
    y: u32,
    z: u32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(
            Entity::from_bits(entity),
            name,
            ShaderValue::UInt3([x, y, z]),
        )
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_uvec4(
    entity: u64,
    name: *const std::ffi::c_char,
    x: u32,
    y: u32,
    z: u32,
    w: u32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(
            Entity::from_bits(entity),
            name,
            ShaderValue::UInt4([x, y, z, w]),
        )
    });
}

/// # Safety
/// - `name` must be non-null.
/// - `value` must point to at least 16 f32 elements (column-major).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_mat4(
    entity: u64,
    name: *const std::ffi::c_char,
    value: *const f32,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        // SAFETY: caller guarantees 16 valid f32 elements
        let m: [f32; 16] = unsafe { std::slice::from_raw_parts(value, 16) }
            .try_into()
            .unwrap();
        shader_set(Entity::from_bits(entity), name, ShaderValue::Mat4(m))
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_texture(
    entity: u64,
    name: *const std::ffi::c_char,
    image_id: u64,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(
            Entity::from_bits(entity),
            name,
            ShaderValue::Texture(Entity::from_bits(image_id)),
        )
    });
}

/// # Safety
/// - `name` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_shader_set_buffer(
    entity: u64,
    name: *const std::ffi::c_char,
    buf_id: u64,
) {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        shader_set(
            Entity::from_bits(entity),
            name,
            ShaderValue::Buffer(Entity::from_bits(buf_id)),
        )
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_compute_dispatch(compute_id: u64, x: u32, y: u32, z: u32) {
    error::clear_error();
    error::check(|| compute_dispatch(Entity::from_bits(compute_id), x, y, z));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_compute_destroy(compute_id: u64) {
    error::clear_error();
    error::check(|| compute_destroy(Entity::from_bits(compute_id)));
}

/// # Safety
/// - `attr_ids` is valid for `attr_count` u64 reads.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_particles_create(
    capacity: u32,
    attr_ids: *const u64,
    attr_count: u32,
) -> u64 {
    error::clear_error();
    let attrs = if attr_count > 0 && !attr_ids.is_null() {
        unsafe { std::slice::from_raw_parts(attr_ids, attr_count as usize) }
            .iter()
            .map(|&id| Entity::from_bits(id))
            .collect()
    } else {
        vec![]
    };
    error::check(|| particles_create(capacity, attrs))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// # Safety
/// - `attr_ids` is valid for `attr_count` u64 reads.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_particles_create_from_geometry(
    geo_id: u64,
    attr_ids: *const u64,
    attr_count: u32,
) -> u64 {
    error::clear_error();
    let attrs = if attr_count > 0 && !attr_ids.is_null() {
        unsafe { std::slice::from_raw_parts(attr_ids, attr_count as usize) }
            .iter()
            .map(|&id| Entity::from_bits(id))
            .collect()
    } else {
        vec![]
    };
    error::check(|| particles_create_from_geometry(Entity::from_bits(geo_id), attrs))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_destroy(particles_id: u64) {
    error::clear_error();
    error::check(|| particles_destroy(Entity::from_bits(particles_id)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_capacity(particles_id: u64) -> u32 {
    error::clear_error();
    error::check(|| particles_capacity(Entity::from_bits(particles_id))).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_buffer(particles_id: u64, attr_id: u64) -> u64 {
    error::clear_error();
    error::check(|| particles_buffer(Entity::from_bits(particles_id), Entity::from_bits(attr_id)))
        .flatten()
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// # Safety
/// - `attr_ids` is valid for `attr_count` u64 reads.
/// - `attr_byte_lengths` is valid for `attr_count` u64 reads.
/// - `data` is valid for `sum(attr_byte_lengths)` byte reads.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_particles_emit(
    particles_id: u64,
    n: u32,
    attr_ids: *const u64,
    data: *const u8,
    attr_byte_lengths: *const u64,
    attr_count: u32,
) {
    error::clear_error();
    error::check(|| {
        if attr_count == 0 {
            return particles_emit(Entity::from_bits(particles_id), n, vec![]);
        }
        let ids = unsafe { std::slice::from_raw_parts(attr_ids, attr_count as usize) };
        let lens = unsafe { std::slice::from_raw_parts(attr_byte_lengths, attr_count as usize) };
        let mut offset: usize = 0;
        let mut attribute_data = Vec::with_capacity(attr_count as usize);
        for i in 0..attr_count as usize {
            let len = lens[i] as usize;
            let bytes = unsafe { std::slice::from_raw_parts(data.add(offset), len) }.to_vec();
            attribute_data.push((Entity::from_bits(ids[i]), bytes));
            offset += len;
        }
        particles_emit(Entity::from_bits(particles_id), n, attribute_data)
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_emit_gpu(particles_id: u64, n: u32, compute_id: u64) {
    error::clear_error();
    error::check(|| {
        particles_emit_gpu(
            Entity::from_bits(particles_id),
            n,
            Entity::from_bits(compute_id),
        )
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_noise() -> u64 {
    error::clear_error();
    error::check(particles_kernel_noise)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_transform() -> u64 {
    error::clear_error();
    error::check(particles_kernel_transform)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_attract() -> u64 {
    error::clear_error();
    error::check(particles_kernel_attract)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_drag() -> u64 {
    error::clear_error();
    error::check(particles_kernel_drag)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_vortex() -> u64 {
    error::clear_error();
    error::check(particles_kernel_vortex)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_attribute_add(particles_id: u64, attribute_id: u64) -> i32 {
    error::clear_error();
    error::check(|| {
        particles_attribute_add(
            Entity::from_bits(particles_id),
            Entity::from_bits(attribute_id),
            None,
        )
    })
    .map(|_| 0)
    .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_force() -> u64 {
    error::clear_error();
    error::check(particles_kernel_force)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_integrate() -> u64 {
    error::clear_error();
    error::check(particles_kernel_integrate)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_age() -> u64 {
    error::clear_error();
    error::check(particles_kernel_age)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_bounds_sphere() -> u64 {
    error::clear_error();
    error::check(particles_kernel_bounds_sphere)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_bounds_box() -> u64 {
    error::clear_error();
    error::check(particles_kernel_bounds_box)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_bounds_geometry(geometry_entity: u64) -> u64 {
    error::clear_error();
    error::check(|| particles_kernel_bounds_geometry(Entity::from_bits(geometry_entity)))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_impulse() -> u64 {
    error::clear_error();
    error::check(particles_kernel_impulse)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_flock() -> u64 {
    error::clear_error();
    error::check(particles_kernel_flock)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_orient() -> u64 {
    error::clear_error();
    error::check(particles_kernel_orient)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_field() -> u64 {
    error::clear_error();
    error::check(particles_kernel_field)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_attr_linear() -> u64 {
    error::clear_error();
    error::check(particles_kernel_attr_linear)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_attr_combine() -> u64 {
    error::clear_error();
    error::check(particles_kernel_attr_combine)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_attr_mix() -> u64 {
    error::clear_error();
    error::check(particles_kernel_attr_mix)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_attr_lookup1d() -> u64 {
    error::clear_error();
    error::check(particles_kernel_attr_lookup1d)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_kernel_attr_lookup2d() -> u64 {
    error::clear_error();
    error::check(particles_kernel_attr_lookup2d)
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_scatter_create(geometry_id: u64) -> u64 {
    error::clear_error();
    error::check(|| particles_scatter_create(Entity::from_bits(geometry_id)))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_scatter_volume_create(geometry_id: u64) -> u64 {
    error::clear_error();
    error::check(|| particles_scatter_volume_create(Entity::from_bits(geometry_id)))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// # Safety
/// - `path` is a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_gltf_load(
    graphics_id: u64,
    path: *const std::ffi::c_char,
) -> u64 {
    error::clear_error();
    error::check(|| {
        let path = unsafe { cstr_to_str(path) }?;
        gltf_load(Entity::from_bits(graphics_id), path)
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

/// # Safety
/// - `name` is a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_gltf_geometry(
    gltf_id: u64,
    name: *const std::ffi::c_char,
) -> u64 {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        gltf_geometry(Entity::from_bits(gltf_id), name)
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

/// # Safety
/// - `name` is a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_gltf_material(
    gltf_id: u64,
    name: *const std::ffi::c_char,
) -> u64 {
    error::clear_error();
    error::check(|| {
        let name = unsafe { cstr_to_str(name) }?;
        gltf_material(Entity::from_bits(gltf_id), name)
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_gltf_camera(gltf_id: u64, index: u32) {
    error::clear_error();
    error::check(|| gltf_camera(Entity::from_bits(gltf_id), index as usize));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_gltf_light(gltf_id: u64, index: u32) -> u64 {
    error::clear_error();
    error::check(|| gltf_light(Entity::from_bits(gltf_id), index as usize))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_apply(particles_id: u64, compute_id: u64) {
    error::clear_error();
    error::check(|| {
        particles_apply(
            Entity::from_bits(particles_id),
            Entity::from_bits(compute_id),
        )
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_flock(particles_id: u64, flock_id: u64) {
    error::clear_error();
    error::check(|| {
        particles_flock_auto(Entity::from_bits(particles_id), Entity::from_bits(flock_id))
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_draw(graphics_id: u64, particles_id: u64, geometry_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Particles {
                particles: Entity::from_bits(particles_id),
                geometry: Some(Entity::from_bits(geometry_id)),
                topology: geometry::Topology::PointList,
            },
        )
    });
}

fn topology_from_u32(topology: u32) -> Result<geometry::Topology, ProcessingError> {
    match topology {
        0 => Ok(geometry::Topology::PointList),
        1 => Ok(geometry::Topology::LineList),
        2 => Ok(geometry::Topology::LineStrip),
        3 => Ok(geometry::Topology::TriangleList),
        4 => Ok(geometry::Topology::TriangleStrip),
        _ => Err(ProcessingError::InvalidArgument(format!(
            "unknown topology {topology}"
        ))),
    }
}

/// Draw `particles` with an explicit topology. `geometry_id` 0 draws raw
/// vertices with the given topology instead of instanced geometry.
#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_draw_topology(
    graphics_id: u64,
    particles_id: u64,
    geometry_id: u64,
    topology: u32,
) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::Particles {
                particles: Entity::from_bits(particles_id),
                geometry: (geometry_id != 0).then(|| Entity::from_bits(geometry_id)),
                topology: topology_from_u32(topology)?,
            },
        )
    });
}

/// Grids are plain structs, not ECS entities, so the FFI keeps them in a
/// handle table.
static GRIDS: std::sync::Mutex<(u64, Option<std::collections::HashMap<u64, Grid>>)> =
    std::sync::Mutex::new((0, None));

fn grid_get(handle: u64) -> Result<Grid, ProcessingError> {
    GRIDS
        .lock()
        .unwrap()
        .1
        .as_ref()
        .and_then(|m| m.get(&handle).copied())
        .ok_or_else(|| ProcessingError::InvalidArgument(format!("unknown grid handle {handle}")))
}

/// Create a spatial hash grid sized to `particles_id`'s capacity. Returns a
/// grid handle (not an entity id), or 0 on error.
#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_grid_create(
    particles_id: u64,
    min_x: f32,
    min_y: f32,
    min_z: f32,
    cell_size: f32,
    dims_x: u32,
    dims_y: u32,
    dims_z: u32,
) -> u64 {
    error::clear_error();
    error::check(|| {
        let capacity = particles_capacity(Entity::from_bits(particles_id))?;
        let grid = grid_create(
            GridParams {
                min: [min_x, min_y, min_z],
                cell_size,
                dims: [dims_x, dims_y, dims_z],
            },
            capacity,
        )?;
        let mut guard = GRIDS.lock().unwrap();
        guard.0 += 1;
        let handle = guard.0;
        guard.1.get_or_insert_with(Default::default).insert(handle, grid);
        Ok(handle)
    })
    .unwrap_or(0)
}

/// Rebuild the grid's cell index from a position buffer.
#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_grid_build(grid_handle: u64, position_buf_id: u64) {
    error::clear_error();
    error::check(|| grid_build(&grid_get(grid_handle)?, Entity::from_bits(position_buf_id)));
}

/// Bind the grid's offsets/sorted buffers and domain uniforms onto `compute`.
#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_grid_bind(grid_handle: u64, compute_id: u64) {
    error::clear_error();
    error::check(|| grid_bind(&grid_get(grid_handle)?, Entity::from_bits(compute_id)));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_grid_destroy(grid_handle: u64) {
    error::clear_error();
    error::check(|| {
        let grid = GRIDS
            .lock()
            .unwrap()
            .1
            .as_mut()
            .and_then(|m| m.remove(&grid_handle))
            .ok_or_else(|| {
                ProcessingError::InvalidArgument(format!("unknown grid handle {grid_handle}"))
            })?;
        buffer_destroy(grid.offsets)?;
        buffer_destroy(grid.cursor)?;
        buffer_destroy(grid.sorted)
    });
}

/// Create a dynamic-topology primitives target over `particles_id`.
/// `topology` must be 0 (points), 1 (lines), or 3 (triangles); capacity is
/// counted in primitives. Returns the target entity id, or 0 on error.
#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_primitives_create(
    particles_id: u64,
    topology: u32,
    capacity_prims: u32,
) -> u64 {
    error::clear_error();
    error::check(|| {
        particles_primitives_create(
            Entity::from_bits(particles_id),
            topology_from_u32(topology)?,
            capacity_prims,
        )
    })
    .map(|e| e.to_bits())
    .unwrap_or(0)
}

/// Apply `compute` over the target's source field with the target's buffers
/// bound under the `processing::prims` reserved names.
#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_primitives_apply(target_id: u64, compute_id: u64) {
    error::clear_error();
    error::check(|| {
        particles_primitives_apply(Entity::from_bits(target_id), Entity::from_bits(compute_id))
    });
}

/// The internal particle field holding the target's expanded vertices (what
/// gets drawn), or 0 on error.
#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_primitives_field(target_id: u64) -> u64 {
    error::clear_error();
    error::check(|| particles_primitives_field(Entity::from_bits(target_id)))
        .map(|e| e.to_bits())
        .unwrap_or(0)
}

/// Primitives the kernels attempted to add this frame, including any dropped
/// for capacity. Reads a small stat back from the GPU.
#[unsafe(no_mangle)]
pub extern "C" fn processing_particles_primitives_attempted(target_id: u64) -> u32 {
    error::clear_error();
    error::check(|| particles_primitives_attempted(Entity::from_bits(target_id))).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_fill_buffer(graphics_id: u64, buffer_id: u64) {
    error::clear_error();
    let graphics_entity = Entity::from_bits(graphics_id);
    error::check(|| {
        graphics_record_command(
            graphics_entity,
            DrawCommand::FillBuffer(Entity::from_bits(buffer_id)),
        )
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_set_albedo_buffer(mat_id: u64, buffer_id: u64) {
    error::clear_error();
    error::check(|| {
        material_set_albedo_buffer(Entity::from_bits(mat_id), Entity::from_bits(buffer_id))
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_material_set_emissive_buffer(mat_id: u64, buffer_id: u64) {
    error::clear_error();
    error::check(|| {
        material_set_emissive_buffer(Entity::from_bits(mat_id), Entity::from_bits(buffer_id))
    });
}

/// # Safety
/// - `out_x`, `out_y`, `out_z` are each valid for one f32 write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_graphics_world_from_screen(
    graphics_id: u64,
    sx: f32,
    sy: f32,
    depth: f32,
    out_x: *mut f32,
    out_y: *mut f32,
    out_z: *mut f32,
) {
    error::clear_error();
    if let Some(world) =
        error::check(|| graphics_world_from_screen(Entity::from_bits(graphics_id), sx, sy, depth))
    {
        unsafe {
            *out_x = world.x;
            *out_y = world.y;
            *out_z = world.z;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_graphics_set_bloom(graphics_id: u64, intensity: f32, threshold: f32) {
    error::clear_error();
    error::check(|| graphics_set_bloom(Entity::from_bits(graphics_id), intensity, threshold));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_graphics_remove_bloom(graphics_id: u64) {
    error::clear_error();
    error::check(|| graphics_remove_bloom(Entity::from_bits(graphics_id)));
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
                return Err(ProcessingError::InvalidArgument(format!(
                    "invalid mouse button: {button}"
                )));
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
        // Silently skip keys with no mapping (media keys, GLFW_KEY_UNKNOWN):
        // matches the Rust GLFW runner's `if let Some(kc) = glfw_key_to_bevy`.
        // An input callback must never take down the render loop.
        let Ok(kc) = key_code_from_u32(key_code) else {
            return Ok(());
        };
        input_set_key(Entity::from_bits(surface_id), kc, pressed)
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_char(surface_id: u64, key_code: u32, codepoint: u32) {
    error::clear_error();
    error::check(|| {
        // 0 = no associated key: char events arrive separately from key
        // events on GLFW. Matches the Rust GLFW runner, which sends
        // KeyCode::Unidentified for WindowEvent::Char.
        let kc = if key_code == 0 {
            KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified)
        } else {
            key_code_from_u32(key_code)?
        };
        let ch = char::from_u32(codepoint).ok_or_else(|| {
            ProcessingError::InvalidArgument(format!("invalid codepoint: {codepoint}"))
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

pub const PROCESSING_CURSOR_ARROW: u8 = 0;
pub const PROCESSING_CURSOR_CROSS: u8 = 1;
pub const PROCESSING_CURSOR_HAND: u8 = 2;
pub const PROCESSING_CURSOR_MOVE: u8 = 3;
pub const PROCESSING_CURSOR_TEXT: u8 = 4;
pub const PROCESSING_CURSOR_WAIT: u8 = 5;

fn cursor_icon(kind: u8) -> bevy::window::SystemCursorIcon {
    use bevy::window::SystemCursorIcon;
    match kind {
        PROCESSING_CURSOR_CROSS => SystemCursorIcon::Crosshair,
        PROCESSING_CURSOR_HAND => SystemCursorIcon::Pointer,
        PROCESSING_CURSOR_MOVE => SystemCursorIcon::Move,
        PROCESSING_CURSOR_TEXT => SystemCursorIcon::Text,
        PROCESSING_CURSOR_WAIT => SystemCursorIcon::Wait,
        _ => SystemCursorIcon::Default,
    }
}

/// Show the mouse cursor with the given system type (PROCESSING_CURSOR_*).
#[unsafe(no_mangle)]
pub extern "C" fn processing_cursor(surface_id: u64, kind: u8) {
    error::clear_error();
    let surface = Entity::from_bits(surface_id);
    error::check(|| {
        input_set_cursor_visible(surface, true)?;
        input_set_cursor_icon(surface, cursor_icon(kind))
    });
}

/// Hide the mouse cursor.
#[unsafe(no_mangle)]
pub extern "C" fn processing_no_cursor(surface_id: u64) {
    error::clear_error();
    error::check(|| input_set_cursor_visible(Entity::from_bits(surface_id), false));
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_input_flush() {
    error::clear_error();
    error::check(input_flush);
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
    error::check(input_mouse_is_pressed).unwrap_or(false)
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
    error::check(input_key_is_pressed).unwrap_or(false)
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
pub extern "C" fn processing_key_just_pressed(key_code: u32) -> bool {
    error::clear_error();
    error::check(|| {
        let kc = key_code_from_u32(key_code)?;
        input_key_just_pressed(kc)
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
    error::check(|| input_key_code().map(|opt| opt.map(key_code_to_u32).unwrap_or(0))).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_moved_x() -> f32 {
    error::clear_error();
    error::check(input_moved_x).unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_moved_y() -> f32 {
    error::clear_error();
    error::check(input_moved_y).unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn processing_mouse_wheel() -> f32 {
    error::clear_error();
    error::check(input_mouse_wheel).unwrap_or(0.0)
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
        _ => Err(ProcessingError::InvalidArgument(format!(
            "unknown key code: {val}"
        ))),
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
