#![allow(clippy::module_inception)]

pub mod camera;
pub mod color;
pub mod compute;
pub mod geometry;
pub mod gltf;
pub mod graphics;
pub mod image;
pub mod light;
pub mod material;
pub mod monitor;
pub mod particles;
pub mod render;
pub mod shader_value;
pub mod sketch;
pub mod surface;
pub mod text;
pub mod time;
pub mod transform;

use std::path::PathBuf;

use bevy::{
    asset::AssetEventSystems,
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
};
use processing_core::app_mut;
use processing_core::config::*;
use processing_core::error;

use crate::geometry::{AttributeFormat, AttributeValue};
use crate::graphics::flush;
use crate::image::gpu_image;
use crate::render::command::DrawCommand;

#[derive(Component)]
pub struct Flush;

pub struct ProcessingRenderPlugin;

impl Plugin for ProcessingRenderPlugin {
    fn build(&self, app: &mut App) {
        use render::material::{add_custom_materials, add_processing_materials};
        use render::{activate_cameras, clear_transient_meshes, flush_draw_commands};

        let config = app.world().resource::<Config>().clone();

        app.init_resource::<time::ProcessingFrame>();

        let has_sketch_file = config
            .get(ConfigKey::SketchFileName)
            .is_some_and(|f| !f.is_empty());
        if has_sketch_file {
            app.add_plugins(sketch::LivecodePlugin);
        }

        app.add_plugins((
            image::ImagePlugin,
            graphics::GraphicsPlugin,
            surface::SurfacePlugin,
            geometry::GeometryPlugin,
            light::LightPlugin,
            material::ProcessingMaterialPlugin,
            bevy::pbr::wireframe::WireframePlugin::default(),
            material::custom::CustomMaterialPlugin,
            compute::ComputePlugin,
            particles::ParticlesPlugin,
            camera::OrbitCameraPlugin,
            bevy::camera_controller::free_camera::FreeCameraPlugin,
            bevy::camera_controller::pan_camera::PanCameraPlugin,
            text::font::TextPlugin,
        ));

        app.add_systems(First, (clear_transient_meshes, activate_cameras))
            .add_systems(
                Update,
                (
                    flush_draw_commands,
                    add_processing_materials,
                    add_custom_materials,
                    particles::material::add_particles_materials,
                )
                    .chain()
                    .before(AssetEventSystems),
            );
    }
}

/// Create a WebGPU surface from a macOS NSWindow handle.
#[cfg(target_os = "macos")]
pub fn surface_create_macos(
    window_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                surface::create_surface_macos,
                (window_handle, width, height, scale_factor),
            )
            .unwrap()
    })
}

/// Create a WebGPU surface from a Windows HWND handle.
#[cfg(target_os = "windows")]
pub fn surface_create_windows(
    window_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                surface::create_surface_windows,
                (window_handle, width, height, scale_factor),
            )
            .unwrap()
    })
}

/// Create a WebGPU surface from a Wayland window and display handle.
#[cfg(all(target_os = "linux", feature = "wayland"))]
pub fn surface_create_wayland(
    window_handle: u64,
    display_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                surface::create_surface_wayland,
                (window_handle, display_handle, width, height, scale_factor),
            )
            .unwrap()
    })
}

/// Create a WebGPU surface from an X11 window and display handle.
#[cfg(all(target_os = "linux", feature = "x11"))]
pub fn surface_create_x11(
    window_handle: u64,
    display_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                surface::create_surface_x11,
                (window_handle, display_handle, width, height, scale_factor),
            )
            .unwrap()
    })
}

/// Create a WebGPU surface on Linux, auto-detecting Wayland vs. X11 from the
/// environment.
#[cfg(target_os = "linux")]
pub fn surface_create_linux(
    window_handle: u64,
    display_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> error::Result<Entity> {
    // prefer wayland, since x11 may also be available under xwayland
    let nonempty = |name| std::env::var_os(name).is_some_and(|v| !v.is_empty());
    let is_wayland = nonempty("WAYLAND_DISPLAY") || nonempty("WAYLAND_SOCKET");

    #[cfg(all(feature = "wayland", feature = "x11"))]
    {
        if is_wayland {
            surface_create_wayland(window_handle, display_handle, width, height, scale_factor)
        } else {
            surface_create_x11(window_handle, display_handle, width, height, scale_factor)
        }
    }
    #[cfg(all(feature = "wayland", not(feature = "x11")))]
    {
        let _ = is_wayland;
        surface_create_wayland(window_handle, display_handle, width, height, scale_factor)
    }
    #[cfg(all(not(feature = "wayland"), feature = "x11"))]
    {
        let _ = is_wayland;
        surface_create_x11(window_handle, display_handle, width, height, scale_factor)
    }
    #[cfg(not(any(feature = "wayland", feature = "x11")))]
    {
        let _ = (
            window_handle,
            display_handle,
            width,
            height,
            scale_factor,
            is_wayland,
        );
        Err(processing_core::error::ProcessingError::InvalidArgument(
            "libprocessing was built without `wayland` or `x11` features".into(),
        ))
    }
}

/// Create a WebGPU surface from a web canvas element pointer.
#[cfg(target_arch = "wasm32")]
pub fn surface_create_web(
    window_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                surface::create_surface_web,
                (window_handle, width, height, scale_factor),
            )
            .unwrap()
    })
}

pub fn surface_create_offscreen(
    width: u32,
    height: u32,
    scale_factor: f32,
    texture_format: TextureFormat,
) -> error::Result<Entity> {
    app_mut(|app| {
        let (size, data, texture_format) =
            surface::prepare_offscreen(width, height, scale_factor, texture_format)?;
        let world = app.world_mut();
        let image_entity = world
            .run_system_cached_with(image::create, (size, data, texture_format))
            .unwrap();
        world.entity_mut(image_entity).insert(surface::Surface);
        Ok(image_entity)
    })
}

/// Create a WebGPU surface from a canvas element ID
#[cfg(target_arch = "wasm32")]
pub fn surface_create_from_canvas(
    canvas_id: &str,
    width: u32,
    height: u32,
) -> error::Result<Entity> {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlCanvasElement;

    // find the canvas element
    let web_window = web_sys::window().ok_or(error::ProcessingError::InvalidWindowHandle)?;
    let document = web_window
        .document()
        .ok_or(error::ProcessingError::InvalidWindowHandle)?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or(error::ProcessingError::InvalidWindowHandle)?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| error::ProcessingError::InvalidWindowHandle)?;

    // box and leak the canvas to ensure the pointer remains valid
    // TODO: this is maybe gross, let's find a better way to manage the lifetime
    let canvas_box = Box::new(canvas);
    let canvas_ptr = Box::into_raw(canvas_box) as u64;

    // TODO: not sure if this is right to force here
    let scale_factor = 1.0;

    surface_create_web(canvas_ptr, width, height, scale_factor)
}

pub fn surface_destroy(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::destroy, graphics_entity)
            .unwrap()
    })
}

/// Update window size when resized.
pub fn surface_resize(graphics_entity: Entity, width: u32, height: u32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::resize, (graphics_entity, width, height))
            .unwrap()
    })
}

pub fn surface_set_pixel_density(entity: Entity, density: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::set_pixel_density, (entity, density))
            .unwrap()
    })
}

pub fn surface_set_title(entity: Entity, title: String) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::set_title, (entity, title))
            .unwrap()
    })
}

pub fn surface_position(entity: Entity) -> error::Result<bevy::math::IVec2> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(surface::position, entity)
            .unwrap())
    })
}

pub fn surface_set_position(entity: Entity, x: i32, y: i32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::set_position, (entity, x, y))
            .unwrap()
    })
}

pub fn surface_set_visible(entity: Entity, visible: bool) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::set_visible, (entity, visible))
            .unwrap()
    })
}

pub fn surface_set_resizable(entity: Entity, resizable: bool) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::set_resizable, (entity, resizable))
            .unwrap()
    })
}

pub fn surface_set_decorated(entity: Entity, decorated: bool) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::set_decorated, (entity, decorated))
            .unwrap()
    })
}

pub fn surface_set_window_level(
    entity: Entity,
    level: bevy::window::WindowLevel,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::set_window_level, (entity, level))
            .unwrap()
    })
}

pub fn surface_set_window_mode(
    entity: Entity,
    mode: bevy::window::WindowMode,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::set_window_mode, (entity, mode))
            .unwrap()
    })
}

pub fn surface_set_opacity(entity: Entity, opacity: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::set_opacity, (entity, opacity))
            .unwrap()
    })
}

pub fn surface_iconify(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::iconify, entity)
            .unwrap()
    })
}

pub fn surface_restore(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::restore, entity)
            .unwrap()
    })
}

pub fn surface_maximize(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::maximize, entity)
            .unwrap()
    })
}

pub fn surface_focus(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::focus, entity)
            .unwrap()
    })
}

pub fn surface_position_on_monitor(
    surface: Entity,
    monitor: Entity,
    x: i32,
    y: i32,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::position_on_monitor, (surface, monitor, x, y))
            .unwrap()
    })
}

pub fn surface_center_on_monitor(surface: Entity, monitor: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(surface::center_on_monitor, (surface, monitor))
            .unwrap()
    })
}

/// Create a new graphics surface for rendering.
pub fn graphics_create(
    surface_entity: Entity,
    width: u32,
    height: u32,
    texture_format: TextureFormat,
) -> error::Result<Entity> {
    app_mut(|app| -> error::Result<Entity> {
        let entity = app
            .world_mut()
            .run_system_cached_with(
                graphics::create,
                (width, height, surface_entity, texture_format),
            )
            .unwrap()?;

        graphics::warmup(app, entity)?;
        Ok(entity)
    })
}

/// Begin a new draw pass for the graphics surface.
pub fn graphics_begin_draw(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(graphics::begin_draw, graphics_entity)
            .unwrap()
    })
}

/// Flush current pending draw commands to the graphics surface.
pub fn graphics_flush(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| graphics::flush(app, graphics_entity))
}

/// Present the current frame to the surface.
pub fn graphics_present(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| graphics::present(app, graphics_entity))
}

/// End the current draw pass for the graphics surface.
pub fn graphics_end_draw(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| graphics::end_draw(app, graphics_entity))
}

/// Destroy the graphics surface and free its resources.
pub fn graphics_destroy(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(graphics::destroy, graphics_entity)
            .unwrap()
    })
}

/// Read back raw pixel data from the graphics surface.
pub fn graphics_readback_raw(graphics_entity: Entity) -> error::Result<graphics::ReadbackData> {
    app_mut(|app| {
        graphics::flush(app, graphics_entity)?;
        let vt = graphics::view_target(app, graphics_entity)?;
        let texture = vt.main_texture().clone();
        app.world_mut()
            .run_system_cached_with(graphics::readback_raw, (graphics_entity, texture))
            .unwrap()
    })
}

/// Read back pixel data from the graphics surface as LinearRgba.
pub fn graphics_readback(graphics_entity: Entity) -> error::Result<Vec<LinearRgba>> {
    let raw = graphics_readback_raw(graphics_entity)?;
    let px_size = image::pixel_size(raw.format)?;
    let padded_bytes_per_row = raw.width as usize * px_size;
    image::bytes_to_pixels(
        &raw.bytes,
        raw.format,
        raw.width,
        raw.height,
        padded_bytes_per_row,
    )
}

/// Update the graphics surface with new pixel data.
pub fn graphics_update(graphics_entity: Entity, pixels: &[LinearRgba]) -> error::Result<()> {
    app_mut(|app| {
        let vt = graphics::view_target(app, graphics_entity)?;
        let texture = vt.main_texture().clone();
        let world = app.world_mut();
        let size = world
            .get::<graphics::Graphics>(graphics_entity)
            .ok_or(error::ProcessingError::GraphicsNotFound)?
            .size;
        let (data, px_size) = graphics::prepare_update_region(
            world,
            graphics_entity,
            size.width,
            size.height,
            pixels,
        )?;
        world
            .run_system_cached_with(
                graphics::update_region_write,
                (
                    graphics_entity,
                    texture,
                    0,
                    0,
                    size.width,
                    size.height,
                    data,
                    px_size,
                ),
            )
            .unwrap()
    })
}

/// Update a region of the graphics surface with new pixel data.
pub fn graphics_update_region(
    graphics_entity: Entity,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    pixels: &[LinearRgba],
) -> error::Result<()> {
    app_mut(|app| {
        let vt = graphics::view_target(app, graphics_entity)?;
        let texture = vt.main_texture().clone();
        let world = app.world_mut();
        let (data, px_size) =
            graphics::prepare_update_region(world, graphics_entity, width, height, pixels)?;
        world
            .run_system_cached_with(
                graphics::update_region_write,
                (graphics_entity, texture, x, y, width, height, data, px_size),
            )
            .unwrap()
    })
}

/// Set the color mode for a graphics entity.
pub fn graphics_set_color_mode(
    graphics_entity: Entity,
    mode: color::ColorMode,
) -> error::Result<()> {
    app_mut(|app| {
        let mut entity = app
            .world_mut()
            .get_entity_mut(graphics_entity)
            .map_err(|_| error::ProcessingError::GraphicsNotFound)?;
        if let Some(mut cm) = entity.get_mut::<color::ColorMode>() {
            *cm = mode;
        }
        Ok(())
    })
}

/// Get the color mode for a graphics entity.
pub fn graphics_get_color_mode(graphics_entity: Entity) -> error::Result<color::ColorMode> {
    app_mut(|app| {
        app.world()
            .get::<color::ColorMode>(graphics_entity)
            .copied()
            .ok_or(error::ProcessingError::GraphicsNotFound)
    })
}

/// Record a drawing command for a window
pub fn graphics_record_command(graphics_entity: Entity, cmd: DrawCommand) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(graphics::record_command, (graphics_entity, cmd))
            .unwrap()
    })
}

pub fn graphics_mode_3d(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        flush(app, graphics_entity)?;
        app.world_mut()
            .run_system_cached_with(graphics::mode_3d, graphics_entity)
            .unwrap()
    })
}

pub fn graphics_mode_2d(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        flush(app, graphics_entity)?;
        app.world_mut()
            .run_system_cached_with(graphics::mode_2d, graphics_entity)
            .unwrap()
    })
}

pub fn graphics_orbit_camera(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::enable_orbit_camera, graphics_entity)
            .unwrap()
    })
}

pub fn graphics_free_camera(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::enable_free_camera, graphics_entity)
            .unwrap()
    })
}

pub fn graphics_pan_camera(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::enable_pan_camera, graphics_entity)
            .unwrap()
    })
}

pub fn graphics_disable_camera_controller(graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::disable_camera_controller, graphics_entity)
            .unwrap()
    })
}

pub fn camera_set_distance(entity: Entity, distance: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::set_distance, (entity, distance))
            .unwrap()
    })
}

pub fn camera_set_center(entity: Entity, center: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::set_center, (entity, center))
            .unwrap()
    })
}

pub fn camera_set_min_distance(entity: Entity, min: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::set_min_distance, (entity, min))
            .unwrap()
    })
}

pub fn camera_set_max_distance(entity: Entity, max: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::set_max_distance, (entity, max))
            .unwrap()
    })
}

pub fn camera_set_speed(entity: Entity, speed: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::set_speed, (entity, speed))
            .unwrap()
    })
}

pub fn camera_reset(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(camera::reset_camera, entity)
            .unwrap()
    })
}

pub fn graphics_perspective(
    graphics_entity: Entity,
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
    near_clip_plane: Vec4,
) -> error::Result<()> {
    app_mut(|app| {
        flush(app, graphics_entity)?;
        app.world_mut()
            .run_system_cached_with(
                graphics::perspective,
                (
                    graphics_entity,
                    PerspectiveProjection {
                        fov,
                        aspect_ratio,
                        near,
                        far,
                        near_clip_plane,
                    },
                ),
            )
            .unwrap()
    })
}

#[allow(clippy::too_many_arguments)]
pub fn graphics_ortho(
    graphics_entity: Entity,
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> error::Result<()> {
    app_mut(|app| {
        flush(app, graphics_entity)?;
        app.world_mut()
            .run_system_cached_with(
                graphics::ortho,
                (
                    graphics_entity,
                    graphics::OrthoArgs {
                        left,
                        right,
                        bottom,
                        top,
                        near,
                        far,
                    },
                ),
            )
            .unwrap()
    })
}

pub fn transform_set_position(entity: Entity, position: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::set_position, (entity, position))
            .unwrap()
    })
}

pub fn transform_translate(entity: Entity, offset: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::translate, (entity, offset))
            .unwrap()
    })
}

pub fn transform_set_rotation(entity: Entity, euler: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::set_rotation, (entity, euler))
            .unwrap()
    })
}

pub fn transform_rotate_x(entity: Entity, angle: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::rotate_x, (entity, angle))
            .unwrap()
    })
}

pub fn transform_rotate_y(entity: Entity, angle: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::rotate_y, (entity, angle))
            .unwrap()
    })
}

pub fn transform_rotate_z(entity: Entity, angle: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::rotate_z, (entity, angle))
            .unwrap()
    })
}

pub fn transform_rotate_axis(entity: Entity, angle: f32, axis: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::rotate_axis, (entity, angle, axis))
            .unwrap()
    })
}

pub fn transform_set_scale(entity: Entity, scale: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::set_scale, (entity, scale))
            .unwrap()
    })
}

pub fn transform_scale(entity: Entity, factor: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::scale, (entity, factor))
            .unwrap()
    })
}

pub fn transform_look_at(entity: Entity, target: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::look_at, (entity, target))
            .unwrap()
    })
}

pub fn transform_reset(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::reset, entity)
            .unwrap()
    })
}

/// Create a new image with given size and data.
pub fn image_create(
    size: Extent3d,
    data: Vec<u8>,
    texture_format: TextureFormat,
) -> error::Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(image::create, (size, data, texture_format))
            .unwrap())
    })
}

/// Load an image from disk.
#[cfg(not(target_arch = "wasm32"))]
pub fn image_load(path: &str) -> error::Result<Entity> {
    let path = PathBuf::from(path);
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(image::load, path)
            .unwrap()
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn image_load(path: &str) -> error::Result<Entity> {
    use bevy::prelude::{Handle, Image};

    let path = PathBuf::from(path);

    let handle: Handle<Image> = app_mut(|app| Ok(image::load_start(app.world_mut(), path)))?;

    // poll until loaded, yielding to event loop
    loop {
        let is_loaded = app_mut(|app| Ok(image::is_loaded(app.world(), &handle)))?;
        if is_loaded {
            break;
        }

        // yield to let fetch complete
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
                .unwrap();
        }))
        .await
        .unwrap();

        // run an update to process asset events
        app_mut(|app| {
            app.update();
            Ok(())
        })?;
    }

    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(image::from_handle, handle)
            .unwrap()
    })
}

/// Resize an existing image to new size.
pub fn image_resize(entity: Entity, new_size: Extent3d) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(image::resize, (entity, new_size))
            .unwrap()
    })
}

/// Read back image data from GPU to CPU.
pub fn image_readback(entity: Entity) -> error::Result<Vec<LinearRgba>> {
    app_mut(|app| {
        let texture = gpu_image(app, entity)?.texture.clone();
        app.world_mut()
            .run_system_cached_with(image::readback, (entity, texture))
            .unwrap()
    })
}

/// Update an existing image with new pixel data.
pub fn image_update(entity: Entity, pixels: &[LinearRgba]) -> error::Result<()> {
    app_mut(|app| {
        let texture = gpu_image(app, entity)?.texture.clone();
        let world = app.world_mut();
        let size = world
            .get::<image::Image>(entity)
            .ok_or(error::ProcessingError::ImageNotFound)?
            .size;
        let (data, px_size) =
            image::prepare_update_region(world, entity, size.width, size.height, pixels)?;
        world
            .run_system_cached_with(
                image::update_region_write,
                (
                    entity,
                    texture,
                    0,
                    0,
                    size.width,
                    size.height,
                    data,
                    px_size,
                ),
            )
            .unwrap()
    })
}

/// Update a region of an existing image with new pixel data.
pub fn image_update_region(
    entity: Entity,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    pixels: &[LinearRgba],
) -> error::Result<()> {
    app_mut(|app| {
        let texture = gpu_image(app, entity)?.texture.clone();
        let world = app.world_mut();
        let (data, px_size) = image::prepare_update_region(world, entity, width, height, pixels)?;
        world
            .run_system_cached_with(
                image::update_region_write,
                (entity, texture, x, y, width, height, data, px_size),
            )
            .unwrap()
    })
}

/// Set the sampler for an image (filter mode + wrap modes).
pub fn image_set_sampler(entity: Entity, filter: u8, wrap_x: u8, wrap_y: u8) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(image::set_sampler, (entity, filter, wrap_x, wrap_y))
            .unwrap()
    })
}

/// Destroy an existing image and free its resources.
pub fn image_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(image::destroy, entity)
            .unwrap()
    })
}

pub fn light_create_directional(
    graphics_entity: Entity,
    color: Color,
    illuminance: f32,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                light::create_directional,
                (graphics_entity, color, illuminance),
            )
            .unwrap()
    })
}

pub fn light_create_point(
    graphics_entity: Entity,
    color: Color,
    intensity: f32,
    range: f32,
    radius: f32,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                light::create_point,
                (graphics_entity, color, intensity, range, radius),
            )
            .unwrap()
    })
}

pub fn light_create_spot(
    graphics_entity: Entity,
    color: Color,
    intensity: f32,
    range: f32,
    radius: f32,
    inner_angle: f32,
    outer_angle: f32,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                light::create_spot,
                (
                    graphics_entity,
                    color,
                    intensity,
                    range,
                    radius,
                    inner_angle,
                    outer_angle,
                ),
            )
            .unwrap()
    })
}

pub fn geometry_layout_create() -> error::Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(geometry::layout::create, ())
            .unwrap())
    })
}

pub fn geometry_layout_add_position(entity: Entity) -> error::Result<()> {
    app_mut(|app| geometry::layout::add_position(app.world_mut(), entity))
}

pub fn geometry_layout_add_normal(entity: Entity) -> error::Result<()> {
    app_mut(|app| geometry::layout::add_normal(app.world_mut(), entity))
}

pub fn geometry_layout_add_color(entity: Entity) -> error::Result<()> {
    app_mut(|app| geometry::layout::add_color(app.world_mut(), entity))
}

pub fn geometry_layout_add_uv(entity: Entity) -> error::Result<()> {
    app_mut(|app| geometry::layout::add_uv(app.world_mut(), entity))
}

pub fn geometry_layout_add_attribute(
    layout_entity: Entity,
    attr_entity: Entity,
) -> error::Result<()> {
    app_mut(|app| geometry::layout::add_attribute(app.world_mut(), layout_entity, attr_entity))
}

pub fn geometry_layout_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::layout::destroy, entity)
            .unwrap();
        Ok(())
    })
}

pub fn geometry_attribute_create(
    name: impl Into<String>,
    format: AttributeFormat,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::attribute::create, (name.into(), format))
            .unwrap()
    })
}

pub fn geometry_attribute_position() -> Entity {
    app_mut(|app| {
        Ok(app
            .world()
            .resource::<geometry::BuiltinAttributes>()
            .position)
    })
    .unwrap()
}

pub fn geometry_attribute_normal() -> Entity {
    app_mut(|app| Ok(app.world().resource::<geometry::BuiltinAttributes>().normal)).unwrap()
}

pub fn geometry_attribute_color() -> Entity {
    app_mut(|app| Ok(app.world().resource::<geometry::BuiltinAttributes>().color)).unwrap()
}

pub fn geometry_attribute_uv() -> Entity {
    app_mut(|app| Ok(app.world().resource::<geometry::BuiltinAttributes>().uv)).unwrap()
}

pub fn geometry_attribute_rotation() -> Entity {
    app_mut(|app| {
        Ok(app
            .world()
            .resource::<geometry::BuiltinAttributes>()
            .rotation)
    })
    .unwrap()
}

pub fn geometry_attribute_scale() -> Entity {
    app_mut(|app| Ok(app.world().resource::<geometry::BuiltinAttributes>().scale)).unwrap()
}

pub fn geometry_attribute_dead() -> Entity {
    app_mut(|app| Ok(app.world().resource::<geometry::BuiltinAttributes>().dead)).unwrap()
}

pub fn geometry_attribute_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::attribute::destroy, entity)
            .unwrap()?;
        Ok(())
    })
}

pub fn geometry_attribute_info(entity: Entity) -> error::Result<(String, AttributeFormat)> {
    app_mut(|app| {
        let attr = app
            .world()
            .get::<geometry::Attribute>(entity)
            .ok_or(error::ProcessingError::InvalidEntity)?;
        Ok((attr.name.to_string(), attr.format))
    })
}

pub fn geometry_create(topology: geometry::Topology) -> error::Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(geometry::create, topology)
            .unwrap())
    })
}

pub fn geometry_create_with_layout(
    layout_entity: Entity,
    topology: geometry::Topology,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::create_with_layout, (layout_entity, topology))
            .unwrap()
    })
}

pub fn geometry_normal(entity: Entity, normal: Vec3) -> error::Result<()> {
    app_mut(|app| geometry::normal(app.world_mut(), entity, normal))
}

pub fn geometry_color(entity: Entity, color: Vec4) -> error::Result<()> {
    app_mut(|app| geometry::color(app.world_mut(), entity, color))
}

pub fn geometry_uv(entity: Entity, u: f32, v: f32) -> error::Result<()> {
    app_mut(|app| geometry::uv(app.world_mut(), entity, u, v))
}

pub fn geometry_attribute(
    geo_entity: Entity,
    attr_entity: Entity,
    value: AttributeValue,
) -> error::Result<()> {
    app_mut(|app| geometry::attribute(app.world_mut(), geo_entity, attr_entity, value))
}

pub fn geometry_attribute_float(
    geo_entity: Entity,
    attr_entity: Entity,
    v: f32,
) -> error::Result<()> {
    geometry_attribute(geo_entity, attr_entity, AttributeValue::Float(v))
}

pub fn geometry_attribute_float2(
    geo_entity: Entity,
    attr_entity: Entity,
    x: f32,
    y: f32,
) -> error::Result<()> {
    geometry_attribute(geo_entity, attr_entity, AttributeValue::Float2([x, y]))
}

pub fn geometry_attribute_float3(
    geo_entity: Entity,
    attr_entity: Entity,
    x: f32,
    y: f32,
    z: f32,
) -> error::Result<()> {
    geometry_attribute(geo_entity, attr_entity, AttributeValue::Float3([x, y, z]))
}

pub fn geometry_attribute_float4(
    geo_entity: Entity,
    attr_entity: Entity,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) -> error::Result<()> {
    geometry_attribute(
        geo_entity,
        attr_entity,
        AttributeValue::Float4([x, y, z, w]),
    )
}

pub fn geometry_vertex(entity: Entity, position: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::vertex, (entity, position))
            .unwrap()
    })
}

pub fn geometry_index(entity: Entity, i: u32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::index, (entity, i))
            .unwrap()
    })
}

pub fn geometry_vertex_count(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::vertex_count, entity)
            .unwrap()
    })
}

pub fn geometry_index_count(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::index_count, entity)
            .unwrap()
    })
}

pub fn geometry_get_positions(
    entity: Entity,
    start: usize,
    end: usize,
) -> error::Result<Vec<[f32; 3]>> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::get_positions, (entity, start..end))
            .unwrap()
    })
}

pub fn geometry_get_normals(
    entity: Entity,
    start: usize,
    end: usize,
) -> error::Result<Vec<[f32; 3]>> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::get_normals, (entity, start..end))
            .unwrap()
    })
}

pub fn geometry_get_colors(
    entity: Entity,
    start: usize,
    end: usize,
) -> error::Result<Vec<[f32; 4]>> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::get_colors, (entity, start..end))
            .unwrap()
    })
}

pub fn geometry_get_uvs(entity: Entity, start: usize, end: usize) -> error::Result<Vec<[f32; 2]>> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::get_uvs, (entity, start..end))
            .unwrap()
    })
}

pub fn geometry_get_indices(entity: Entity, start: usize, end: usize) -> error::Result<Vec<u32>> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::get_indices, (entity, start..end))
            .unwrap()
    })
}

pub fn geometry_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::destroy, entity)
            .unwrap()
    })
}

pub fn geometry_set_vertex(entity: Entity, index: u32, position: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::set_vertex, (entity, index, position))
            .unwrap()
    })
}

pub fn geometry_set_normal(entity: Entity, index: u32, normal: Vec3) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::set_normal, (entity, index, normal))
            .unwrap()
    })
}

pub fn geometry_set_color(entity: Entity, index: u32, color: Vec4) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::set_color, (entity, index, color))
            .unwrap()
    })
}

pub fn geometry_set_uv(entity: Entity, index: u32, uv: Vec2) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::set_uv, (entity, index, uv))
            .unwrap()
    })
}

pub fn geometry_get_attribute(
    geo_entity: Entity,
    attr_entity: Entity,
    index: u32,
) -> error::Result<AttributeValue> {
    app_mut(|app| {
        let attr = app
            .world()
            .get::<geometry::Attribute>(attr_entity)
            .ok_or(error::ProcessingError::InvalidEntity)?;
        let inner = attr.inner;
        app.world_mut()
            .run_system_cached_with(geometry::get_attribute, (geo_entity, inner, index))
            .unwrap()
    })
}

pub fn geometry_get_attributes(
    geo_entity: Entity,
    attr_entity: Entity,
    start: usize,
    end: usize,
) -> error::Result<Vec<AttributeValue>> {
    app_mut(|app| {
        let attr = app
            .world()
            .get::<geometry::Attribute>(attr_entity)
            .ok_or(error::ProcessingError::InvalidEntity)?;
        let inner = attr.inner;
        app.world_mut()
            .run_system_cached_with(geometry::get_attributes, (geo_entity, inner, start..end))
            .unwrap()
    })
}

pub fn geometry_set_attribute(
    geo_entity: Entity,
    attr_entity: Entity,
    index: u32,
    value: AttributeValue,
) -> error::Result<()> {
    app_mut(|app| {
        let attr = app
            .world()
            .get::<geometry::Attribute>(attr_entity)
            .ok_or(error::ProcessingError::InvalidEntity)?;
        let inner = attr.inner;
        app.world_mut()
            .run_system_cached_with(geometry::set_attribute, (geo_entity, inner, index, value))
            .unwrap()
    })
}

pub fn geometry_create_from_mesh(mesh: Mesh) -> error::Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(geometry::create_from_mesh, mesh)
            .unwrap())
    })
}

pub fn geometry_box(width: f32, height: f32, depth: f32) -> error::Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(geometry::create_box, (width, height, depth))
            .unwrap())
    })
}

pub fn geometry_sphere(radius: f32, sectors: u32, stacks: u32) -> error::Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(geometry::create_sphere, (radius, sectors, stacks))
            .unwrap())
    })
}

/// 3d lattice of `nx * ny * nz` `PointList` vertices centered at the origin,
/// `spacing` units apart. Intended as a position source for
/// [`particles_create_from_geometry`].
pub fn geometry_grid(nx: u32, ny: u32, nz: u32, spacing: f32) -> error::Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(geometry::create_grid, (nx, ny, nz, spacing))
            .unwrap())
    })
}

pub fn poll_for_sketch_updates() -> error::Result<Option<sketch::Sketch>> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached(sketch::sketch_update_handler)
            .unwrap())
    })
}

pub fn shader_create(source: &str) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(material::custom::create_shader, source.to_string())
            .unwrap()
    })
}

/// load a shader. Accepts either an asset-relative path (`"shaders/foo.wgsl"`)
/// or a URL-scheme asset path (`"embedded://crate/file.wgsl"`).
pub fn shader_load(path: &str) -> error::Result<Entity> {
    let path = path.to_string();
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(material::custom::load_shader, path)
            .unwrap()
    })
}

pub fn shader_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(material::custom::destroy_shader, entity)
            .unwrap()
    })
}

pub fn material_create_custom(shader: Entity) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(material::custom::create_custom, shader)
            .unwrap()
    })
}

pub fn material_create_pbr() -> error::Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached(material::create_pbr)
            .unwrap())
    })
}

/// `material_create_pbr` with `unlit = true` set on the base StandardMaterial.
pub fn material_create_unlit() -> error::Result<Entity> {
    let entity = material_create_pbr()?;
    material_set(entity, "unlit", shader_value::ShaderValue::Float(1.0))?;
    Ok(entity)
}

/// set the albedo source to a constant srgba color. If the material is
/// currently buffer-backed, swaps the asset back to plain PBR while
/// preserving every other `StandardMaterial` field.
pub fn material_set_albedo_color(entity: Entity, color: [f32; 4]) -> error::Result<()> {
    use crate::material::ProcessingMaterial;
    use crate::particles::material::ParticlesMaterial;
    use crate::render::material::UntypedMaterial;
    use bevy::pbr::ExtendedMaterial;

    type DefaultMat = ExtendedMaterial<StandardMaterial, ProcessingMaterial>;

    app_mut(|app| {
        let untyped = app
            .world()
            .get::<UntypedMaterial>(entity)
            .ok_or(error::ProcessingError::MaterialNotFound)?
            .0
            .clone();
        let new_color = Color::srgba(color[0], color[1], color[2], color[3]);

        if let Ok(handle) = untyped.clone().try_typed::<DefaultMat>() {
            let mut mats = app.world_mut().resource_mut::<Assets<DefaultMat>>();
            let mat = mats
                .get_mut(&handle)
                .ok_or(error::ProcessingError::MaterialNotFound)?;
            mat.into_inner().base.base_color = new_color;
            return Ok(());
        }

        let Ok(handle) = untyped.try_typed::<ParticlesMaterial>() else {
            return Err(error::ProcessingError::MaterialNotFound);
        };
        let world = app.world_mut();
        let preserved = {
            let mut mats = world.resource_mut::<Assets<ParticlesMaterial>>();
            let mat = mats
                .get(&handle)
                .ok_or(error::ProcessingError::MaterialNotFound)?;
            let mut base = mat.base.clone();
            base.base_color = new_color;
            mats.remove(&handle);
            base
        };
        let new_handle = world
            .resource_mut::<Assets<DefaultMat>>()
            .add(ExtendedMaterial {
                base: preserved,
                extension: ProcessingMaterial { blend_state: None },
            });
        world
            .entity_mut(entity)
            .insert(UntypedMaterial(new_handle.untyped()));
        Ok(())
    })
}

/// set the albedo source to a per-particle color buffer (`Float4` per slot,
/// indexed by `mesh.tag`). Preserves all other `StandardMaterial` fields;
/// `base_color` modulates the buffer color.
pub fn material_set_albedo_buffer(
    entity: Entity,
    color_buffer_entity: Entity,
) -> error::Result<()> {
    use crate::material::ProcessingMaterial;
    use crate::particles::material::{ParticlesExtension, ParticlesMaterial};
    use crate::render::material::UntypedMaterial;
    use bevy::pbr::ExtendedMaterial;

    type DefaultMat = ExtendedMaterial<StandardMaterial, ProcessingMaterial>;

    app_mut(|app| {
        let buffer_handle = app
            .world()
            .get::<compute::Buffer>(color_buffer_entity)
            .ok_or(error::ProcessingError::BufferNotFound)?
            .handle
            .clone();
        let untyped = app
            .world()
            .get::<UntypedMaterial>(entity)
            .ok_or(error::ProcessingError::MaterialNotFound)?
            .0
            .clone();

        if let Ok(handle) = untyped.clone().try_typed::<ParticlesMaterial>() {
            let mut mats = app.world_mut().resource_mut::<Assets<ParticlesMaterial>>();
            let mat = mats
                .get_mut(&handle)
                .ok_or(error::ProcessingError::MaterialNotFound)?;
            mat.into_inner().extension.colors = buffer_handle;
            return Ok(());
        }

        let Ok(handle) = untyped.try_typed::<DefaultMat>() else {
            return Err(error::ProcessingError::MaterialNotFound);
        };
        let world = app.world_mut();
        let preserved = {
            let mut mats = world.resource_mut::<Assets<DefaultMat>>();
            let base = mats
                .get(&handle)
                .ok_or(error::ProcessingError::MaterialNotFound)?
                .base
                .clone();
            mats.remove(&handle);
            base
        };
        let new_handle = world
            .resource_mut::<Assets<ParticlesMaterial>>()
            .add(ExtendedMaterial {
                base: preserved,
                extension: ParticlesExtension {
                    colors: buffer_handle,
                },
            });
        world
            .entity_mut(entity)
            .insert(UntypedMaterial(new_handle.untyped()));
        Ok(())
    })
}

pub fn material_set(
    entity: Entity,
    name: impl Into<String>,
    value: shader_value::ShaderValue,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(material::set_property, (entity, name.into(), value))
            .unwrap()
    })
}

pub fn material_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(material::destroy, entity)
            .unwrap()
    })
}

pub fn surface_focused(entity: Entity) -> error::Result<bool> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(surface::focused, entity)
            .unwrap())
    })
}

pub fn surface_scale_factor(entity: Entity) -> error::Result<f32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(surface::scale_factor, entity)
            .unwrap())
    })
}

pub fn surface_physical_width(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(surface::physical_width, entity)
            .unwrap())
    })
}

pub fn surface_physical_height(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(surface::physical_height, entity)
            .unwrap())
    })
}

pub fn surface_width(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(surface::width, entity)
            .unwrap())
    })
}

pub fn surface_height(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(surface::height, entity)
            .unwrap())
    })
}

pub fn monitor_list() -> error::Result<Vec<Entity>> {
    app_mut(|app| Ok(app.world_mut().run_system_cached(monitor::list).unwrap()))
}

pub fn monitor_primary() -> error::Result<Option<Entity>> {
    app_mut(|app| Ok(app.world_mut().run_system_cached(monitor::primary).unwrap()))
}

pub fn monitor_width(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(monitor::width, entity)
            .unwrap())
    })
}

pub fn monitor_height(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(monitor::height, entity)
            .unwrap())
    })
}

pub fn monitor_scale_factor(entity: Entity) -> error::Result<f64> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(monitor::scale_factor, entity)
            .unwrap())
    })
}

pub fn monitor_refresh_rate_millihertz(entity: Entity) -> error::Result<Option<u32>> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(monitor::refresh_rate_millihertz, entity)
            .unwrap())
    })
}

pub fn monitor_name(entity: Entity) -> error::Result<Option<String>> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(monitor::name, entity)
            .unwrap())
    })
}

pub fn monitor_position(entity: Entity) -> error::Result<bevy::math::IVec2> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(surface::monitor_position, entity)
            .unwrap())
    })
}

pub fn monitor_workarea(entity: Entity) -> error::Result<bevy::math::IRect> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(surface::monitor_workarea, entity)
            .unwrap())
    })
}

pub fn frame_count() -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached(time::frame_count)
            .unwrap())
    })
}

pub fn advance_frame_count() -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached(time::advance_frame_count)
            .unwrap();
        Ok(())
    })
}

pub fn delta_time() -> error::Result<f32> {
    app_mut(|app| Ok(app.world_mut().run_system_cached(time::delta_secs).unwrap()))
}

pub fn elapsed_time() -> error::Result<f32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached(time::elapsed_secs)
            .unwrap())
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn gltf_load(graphics_entity: Entity, path: &str) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(gltf::load, (graphics_entity, path.to_string()))
            .unwrap()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn gltf_geometry(gltf_entity: Entity, name: &str) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(gltf::geometry, (gltf_entity, name.to_string()))
            .unwrap()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn gltf_material(gltf_entity: Entity, name: &str) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(gltf::material, (gltf_entity, name.to_string()))
            .unwrap()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn gltf_mesh_names(gltf_entity: Entity) -> error::Result<Vec<String>> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(gltf::mesh_names, gltf_entity)
            .unwrap()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn gltf_material_names(gltf_entity: Entity) -> error::Result<Vec<String>> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(gltf::material_names, gltf_entity)
            .unwrap()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn gltf_camera(gltf_entity: Entity, index: usize) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(gltf::camera, (gltf_entity, index))
            .unwrap()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn gltf_light(gltf_entity: Entity, index: usize) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(gltf::light, (gltf_entity, index))
            .unwrap()
    })
}

pub fn buffer_create(size: u64) -> error::Result<Entity> {
    app_mut(|app| {
        let entity = app
            .world_mut()
            .run_system_cached_with(compute::create_buffer, size)
            .unwrap();
        app.update();
        Ok(entity)
    })
}

pub fn buffer_create_with_data(data: Vec<u8>) -> error::Result<Entity> {
    app_mut(|app| {
        let entity = app
            .world_mut()
            .run_system_cached_with(compute::create_buffer_with_data, data)
            .unwrap();
        app.update();
        Ok(entity)
    })
}

pub fn buffer_size(entity: Entity) -> error::Result<u64> {
    app_mut(|app| {
        Ok(app
            .world()
            .get::<compute::Buffer>(entity)
            .ok_or(error::ProcessingError::BufferNotFound)?
            .size)
    })
}

pub fn buffer_write(entity: Entity, data: Vec<u8>) -> error::Result<()> {
    buffer_write_range(entity, 0, data, true)
}

pub fn buffer_write_element(entity: Entity, offset: u64, data: Vec<u8>) -> error::Result<()> {
    buffer_write_range(entity, offset, data, false)
}

fn ensure_buffer_synced(app: &mut App, entity: Entity) -> error::Result<()> {
    let (handle, readback_buffer, size, synced) = {
        let buf = app
            .world()
            .get::<compute::Buffer>(entity)
            .ok_or(error::ProcessingError::BufferNotFound)?;
        (
            buf.handle.clone(),
            buf.readback_buffer.clone(),
            buf.size,
            buf.synced,
        )
    };
    if synced {
        return Ok(());
    }
    let bytes = app
        .sub_app_mut(bevy::render::RenderApp)
        .world_mut()
        .run_system_cached_with(
            compute::read_buffer_gpu,
            (handle.clone(), readback_buffer, size),
        )
        .unwrap()?;

    let world = app.world_mut();
    {
        let mut buffers = world.resource_mut::<Assets<bevy::render::storage::ShaderBuffer>>();
        let asset = buffers
            .get_mut_untracked(handle.id())
            .ok_or(error::ProcessingError::BufferNotFound)?;
        asset.data = Some(bytes);
    }

    let mut buf = world
        .get_mut::<compute::Buffer>(entity)
        .ok_or(error::ProcessingError::BufferNotFound)?;
    buf.synced = true;
    Ok(())
}

fn buffer_write_range(
    entity: Entity,
    offset: u64,
    data: Vec<u8>,
    exact_size: bool,
) -> error::Result<()> {
    app_mut(|app| {
        let (handle, size) = {
            let buf = app
                .world()
                .get::<compute::Buffer>(entity)
                .ok_or(error::ProcessingError::BufferNotFound)?;
            (buf.handle.clone(), buf.size)
        };
        let end = offset.checked_add(data.len() as u64).ok_or_else(|| {
            error::ProcessingError::InvalidArgument("offset + len overflow".to_string())
        })?;
        if exact_size && (offset != 0 || end != size) {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "buffer_write data length {} does not match buffer size {size}; \
                 destroy and re-create to resize, or use buffer_write_element for partial writes",
                data.len()
            )));
        }
        if end > size {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "buffer write out of bounds: offset {offset} + len {} > size {size}",
                data.len()
            )));
        }
        ensure_buffer_synced(app, entity)?;
        app.world_mut()
            .run_system_cached_with(compute::write_buffer_cpu, (handle, offset, data))
            .unwrap()
    })
}

pub fn buffer_read_element(entity: Entity, offset: u64, len: u64) -> error::Result<Vec<u8>> {
    buffer_read_range(entity, offset, len)
}

pub fn buffer_read(entity: Entity) -> error::Result<Vec<u8>> {
    let size = buffer_size(entity)?;
    buffer_read_range(entity, 0, size)
}

fn buffer_read_range(entity: Entity, offset: u64, len: u64) -> error::Result<Vec<u8>> {
    app_mut(|app| {
        let size = app
            .world()
            .get::<compute::Buffer>(entity)
            .ok_or(error::ProcessingError::BufferNotFound)?
            .size;
        let end = offset.checked_add(len).ok_or_else(|| {
            error::ProcessingError::InvalidArgument("offset + len overflow".to_string())
        })?;
        if end > size {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "buffer read out of bounds: offset {offset} + len {len} > size {size}"
            )));
        }
        ensure_buffer_synced(app, entity)?;
        let handle = app
            .world()
            .get::<compute::Buffer>(entity)
            .ok_or(error::ProcessingError::BufferNotFound)?
            .handle
            .clone();
        let buffers = app
            .world()
            .resource::<Assets<bevy::render::storage::ShaderBuffer>>();
        let data = buffers
            .get(&handle)
            .and_then(|a| a.data.as_ref())
            .ok_or(error::ProcessingError::BufferNotFound)?;
        Ok(data[offset as usize..(offset + len) as usize].to_vec())
    })
}

pub fn buffer_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(compute::destroy_buffer, entity)
            .unwrap()
    })
}

pub fn compute_create(shader_entity: Entity) -> error::Result<Entity> {
    app_mut(|app| compute::create_compute(app, shader_entity))
}

pub fn compute_set(
    entity: Entity,
    name: impl Into<String>,
    value: shader_value::ShaderValue,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(compute::set_compute_property, (entity, name.into(), value))
            .unwrap()
    })
}

pub fn compute_dispatch(entity: Entity, x: u32, y: u32, z: u32) -> error::Result<()> {
    app_mut(|app| {
        app.update();

        let args = {
            let c = app
                .world()
                .get::<compute::Compute>(entity)
                .ok_or(error::ProcessingError::ComputeNotFound)?;
            (
                c.pipeline_id,
                c.bind_group_layout_descriptors.clone(),
                c.shader.clone(),
                x,
                y,
                z,
            )
        };
        app.sub_app_mut(bevy::render::RenderApp)
            .world_mut()
            .run_system_cached_with(compute::dispatch, args)
            .unwrap()
    })
}

pub fn compute_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(compute::destroy_compute, entity)
            .unwrap()
    })
}

pub fn particles_create(capacity: u32, attribute_entities: Vec<Entity>) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(particles::create, (capacity, attribute_entities))
            .unwrap()
    })
}

/// capacity = `geometry`'s vertex count. Builtin attributes (`position`,
/// `normal`, `color`, `uv`) are seeded from the matching mesh attribute when
/// formats line up; everything else is zero-initialized.
pub fn particles_create_from_geometry(
    geometry_entity: Entity,
    attribute_entities: Vec<Entity>,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                particles::create_from_geometry,
                (geometry_entity, attribute_entities),
            )
            .unwrap()
    })
}

pub fn particles_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(particles::destroy, entity)
            .unwrap()
    })
}

pub fn particles_capacity(entity: Entity) -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world()
            .get::<particles::Particles>(entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?
            .capacity)
    })
}

pub fn particles_buffer(entity: Entity, attribute_entity: Entity) -> error::Result<Option<Entity>> {
    app_mut(|app| {
        Ok(app
            .world()
            .get::<particles::Particles>(entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?
            .buffer(attribute_entity))
    })
}

/// GPU-driven emission into the next `count` ring-buffer slots. Auto-binds
/// attribute buffers (same convention as [`particles_apply`]) and an
/// `emit_range: vec4<f32> = (base_slot, count, capacity, 0)` uniform.
pub fn particles_emit_gpu(
    particles_entity: Entity,
    count: u32,
    compute_entity: Entity,
) -> error::Result<()> {
    if count == 0 {
        return Ok(());
    }
    const WORKGROUP_SIZE: u32 = 64;

    let (capacity, head, buffers) = app_mut(|app| {
        let world = app.world();
        let field = world
            .get::<particles::Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        if count > field.capacity {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "particles_emit_gpu count={} exceeds field capacity {}",
                count, field.capacity
            )));
        }
        let mut buffers: Vec<(String, Entity)> = Vec::with_capacity(field.buffers.len());
        for (&attr_entity, &buf_entity) in &field.buffers {
            let attr = world
                .get::<geometry::Attribute>(attr_entity)
                .ok_or(error::ProcessingError::InvalidEntity)?;
            buffers.push((attr.name.to_string(), buf_entity));
        }
        Ok((field.capacity, field.emit_head, buffers))
    })?;

    for (name, buf_entity) in buffers {
        match compute_set(
            compute_entity,
            name,
            shader_value::ShaderValue::Buffer(buf_entity),
        ) {
            Ok(()) => {}
            Err(error::ProcessingError::UnknownShaderProperty(_)) => {}
            Err(e) => return Err(e),
        }
    }

    match compute_set(
        compute_entity,
        "emit_range",
        shader_value::ShaderValue::Float4([head as f32, count as f32, capacity as f32, 0.0]),
    ) {
        Ok(()) => {}
        Err(error::ProcessingError::UnknownShaderProperty(_)) => {}
        Err(e) => return Err(e),
    }

    let workgroup_count = count.div_ceil(WORKGROUP_SIZE);
    compute_dispatch(compute_entity, workgroup_count, 1, 1)?;

    app_mut(|app| {
        let mut field = app
            .world_mut()
            .get_mut::<particles::Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        field.emit_head = (field.emit_head + count) % field.capacity;
        Ok(())
    })
}

/// CPU-driven emission. Writes per-attribute byte payloads into the next `n`
/// ring-buffer slots. Each entry in `attribute_data` must be exactly
/// `attr.byte_size * n` bytes. On wrap, oldest slots are overwritten.
pub fn particles_emit(
    particles_entity: Entity,
    n: u32,
    attribute_data: Vec<(Entity, Vec<u8>)>,
) -> error::Result<()> {
    if n == 0 {
        return Ok(());
    }

    let (capacity, head, attr_specs) = app_mut(|app| {
        let world = app.world();
        let field = world
            .get::<particles::Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        if n > field.capacity {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "particles_emit n={} exceeds field capacity {}",
                n, field.capacity
            )));
        }
        let mut specs: Vec<(Entity, u32, Entity)> = Vec::with_capacity(attribute_data.len());
        for (attr_entity, _) in &attribute_data {
            let attr = world
                .get::<geometry::Attribute>(*attr_entity)
                .ok_or(error::ProcessingError::InvalidEntity)?;
            let buf = field.buffer(*attr_entity).ok_or_else(|| {
                error::ProcessingError::InvalidArgument(format!(
                    "particles have no buffer for attribute {:?}",
                    attr_entity
                ))
            })?;
            specs.push((*attr_entity, attr.format.byte_size() as u32, buf));
        }
        Ok((field.capacity, field.emit_head, specs))
    })?;

    for ((_, bytes), &(_, byte_size, buf)) in attribute_data.iter().zip(attr_specs.iter()) {
        let expected = (n as usize) * (byte_size as usize);
        if bytes.len() != expected {
            return Err(error::ProcessingError::InvalidArgument(format!(
                "expected {} bytes ({} particles * {} bytes), got {}",
                expected,
                n,
                byte_size,
                bytes.len()
            )));
        }
        let first_chunk_n = (capacity - head).min(n);
        let split = (first_chunk_n as usize) * (byte_size as usize);
        let first_offset = (head as u64) * (byte_size as u64);
        buffer_write_element(buf, first_offset, bytes[..split].to_vec())?;
        if first_chunk_n < n {
            buffer_write_element(buf, 0, bytes[split..].to_vec())?;
        }
    }

    app_mut(|app| {
        let mut field = app
            .world_mut()
            .get_mut::<particles::Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        field.emit_head = (field.emit_head + n) % field.capacity;
        Ok(())
    })
}

/// built-in noise kernel: displaces `position` by 3d value noise. Uniforms:
/// `scale: f32`, `strength: f32`, `time: f32`.
pub fn particles_kernel_noise() -> error::Result<Entity> {
    let shader = shader_load(particles::kernels::NOISE_PATH)?;
    compute_create(shader)
}

/// built-in transform kernel: scale → axis-angle rotate → translate on
/// `position`. Uniforms: `translate: vec3`, `rotation_axis: vec3`,
/// `rotation_angle: f32`, `scale: vec3`. Identity defaults are seeded.
pub fn particles_kernel_transform() -> error::Result<Entity> {
    let shader = shader_load(particles::kernels::TRANSFORM_PATH)?;
    let entity = compute_create(shader)?;
    compute_set(
        entity,
        "translate",
        shader_value::ShaderValue::Float3([0.0; 3]),
    )?;
    compute_set(
        entity,
        "rotation_axis",
        shader_value::ShaderValue::Float3([0.0, 1.0, 0.0]),
    )?;
    compute_set(
        entity,
        "rotation_angle",
        shader_value::ShaderValue::Float(0.0),
    )?;
    compute_set(
        entity,
        "scale",
        shader_value::ShaderValue::Float3([1.0, 1.0, 1.0]),
    )?;
    Ok(entity)
}

/// dispatch `compute_entity` against the [`Particles`]'s buffers. Each buffer
/// is auto-bound by attribute name; undeclared bindings are skipped. Kernels
/// must declare `@workgroup_size(64)`. Set uniforms via `compute_set` first.
pub fn particles_apply(particles_entity: Entity, compute_entity: Entity) -> error::Result<()> {
    const WORKGROUP_SIZE: u32 = 64;

    let (capacity, buffers) = app_mut(|app| {
        let world = app.world();
        let field = world
            .get::<particles::Particles>(particles_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        let mut buffers: Vec<(String, Entity)> = Vec::with_capacity(field.buffers.len());
        for (&attr_entity, &buf_entity) in &field.buffers {
            let attr = world
                .get::<geometry::Attribute>(attr_entity)
                .ok_or(error::ProcessingError::InvalidEntity)?;
            buffers.push((attr.name.to_string(), buf_entity));
        }
        Ok((field.capacity, buffers))
    })?;

    for (name, buf_entity) in buffers {
        match compute_set(
            compute_entity,
            name,
            shader_value::ShaderValue::Buffer(buf_entity),
        ) {
            Ok(()) => {}
            Err(error::ProcessingError::UnknownShaderProperty(_)) => {}
            Err(e) => return Err(e),
        }
    }

    let workgroup_count = capacity.div_ceil(WORKGROUP_SIZE);
    compute_dispatch(compute_entity, workgroup_count, 1, 1)
}

// --- Font API ---

/// Load a font file and return a font entity handle.
///
/// Reading fonts from the filesystem is not available on wasm; callers should
/// register font bytes through another path there.
pub fn font_load(path: &str) -> error::Result<Entity> {
    use text::font::{Font, TextContext};

    #[cfg(target_arch = "wasm32")]
    {
        let _ = path;
        return Err(error::ProcessingError::FontLoadError(
            "loading fonts from a file is not supported on wasm".to_string(),
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let data = std::fs::read(path)
            .map_err(|e| error::ProcessingError::FontLoadError(format!("{}: {}", path, e)))?;

        app_mut(|app| {
            let text_cx = app.world().resource::<TextContext>().clone();
            let family_name = text_cx
                .load_font(data)
                .ok_or(error::ProcessingError::FontLoadError(
                    "Could not determine font family name".to_string(),
                ))?;
            let entity = app.world_mut().spawn(Font { family_name }).id();
            Ok(entity)
        })
    }
}

/// Create a font handle from an existing font family name.
pub fn font_create(name: &str) -> error::Result<Entity> {
    use text::font::{Font, TextContext};

    app_mut(|app| {
        let text_cx = app.world().resource::<TextContext>().clone();
        if !text_cx.has_font(name) {
            return Err(error::ProcessingError::FontNotFound);
        }
        let entity = app
            .world_mut()
            .spawn(Font {
                family_name: name.to_string(),
            })
            .id();
        Ok(entity)
    })
}

/// List all available font family names (system + registered).
pub fn font_list() -> error::Result<Vec<String>> {
    use text::font::TextContext;

    app_mut(|app| {
        let text_cx = app.world().resource::<TextContext>().clone();
        Ok(text_cx.list_fonts())
    })
}

/// Query variable font axes for a loaded font.
pub fn font_variations(font_entity: Entity) -> error::Result<Vec<text::font::FontAxisInfo>> {
    use text::font::TextContext;

    app_mut(|app| {
        let font = app
            .world()
            .get::<text::font::Font>(font_entity)
            .ok_or(error::ProcessingError::InvalidArgument(
                "Invalid font entity".to_string(),
            ))?;
        let family = font.family_name.clone();
        let text_cx = app.world().resource::<TextContext>().clone();
        Ok(text_cx.font_variations(&family))
    })
}

/// Query font metadata for a loaded font.
pub fn font_metadata(font_entity: Entity) -> error::Result<text::font::FontMetadata> {
    use text::font::TextContext;

    app_mut(|app| {
        let font = app
            .world()
            .get::<text::font::Font>(font_entity)
            .ok_or(error::ProcessingError::InvalidArgument(
                "Invalid font entity".to_string(),
            ))?;
        let family = font.family_name.clone();
        let text_cx = app.world().resource::<TextContext>().clone();
        text_cx
            .font_metadata(&family)
            .ok_or(error::ProcessingError::InvalidArgument(
                format!("Font family '{}' not found", family),
            ))
    })
}

// --- Text API ---

pub fn graphics_text_font(
    graphics_entity: Entity,
    font_entity: Option<Entity>,
) -> error::Result<()> {
    graphics_record_command(graphics_entity, DrawCommand::TextFont(font_entity))
}

pub fn graphics_text_style(graphics_entity: Entity, style: u8) -> error::Result<()> {
    use render::command::TextStyle;
    graphics_record_command(graphics_entity, DrawCommand::TextStyle(TextStyle::from(style)))
}

pub fn graphics_text_align(graphics_entity: Entity, h: u8, v: u8) -> error::Result<()> {
    use render::command::{TextAlignH, TextAlignV};
    graphics_record_command(
        graphics_entity,
        DrawCommand::TextAlign {
            h: TextAlignH::from(h),
            v: TextAlignV::from(v),
        },
    )
}

pub fn graphics_text_wrap(graphics_entity: Entity, mode: u8) -> error::Result<()> {
    use render::command::TextWrapMode;
    graphics_record_command(graphics_entity, DrawCommand::TextWrap(TextWrapMode::from(mode)))
}

pub fn graphics_text_weight(graphics_entity: Entity, weight: f32) -> error::Result<()> {
    graphics_record_command(graphics_entity, DrawCommand::TextWeight(weight))
}

fn parse_tag(tag: &str) -> error::Result<[u8; 4]> {
    let bytes = tag.as_bytes();
    if bytes.len() != 4 {
        return Err(error::ProcessingError::InvalidArgument(
            format!("Font tag must be exactly 4 characters, got '{}'", tag),
        ));
    }
    Ok([bytes[0], bytes[1], bytes[2], bytes[3]])
}

pub fn graphics_text_variation(
    graphics_entity: Entity,
    tag: &str,
    value: f32,
) -> error::Result<()> {
    let tag = parse_tag(tag)?;
    graphics_record_command(graphics_entity, DrawCommand::TextVariation { tag, value })
}

pub fn graphics_clear_text_variations(graphics_entity: Entity) -> error::Result<()> {
    graphics_record_command(graphics_entity, DrawCommand::ClearTextVariations)
}

pub fn graphics_text_feature(
    graphics_entity: Entity,
    tag: &str,
    value: u16,
) -> error::Result<()> {
    let tag = parse_tag(tag)?;
    graphics_record_command(graphics_entity, DrawCommand::TextFeature { tag, value })
}

pub fn graphics_no_text_feature(graphics_entity: Entity, tag: &str) -> error::Result<()> {
    let tag = parse_tag(tag)?;
    graphics_record_command(graphics_entity, DrawCommand::NoTextFeature { tag })
}

pub fn graphics_clear_text_features(graphics_entity: Entity) -> error::Result<()> {
    graphics_record_command(graphics_entity, DrawCommand::ClearTextFeatures)
}

pub fn graphics_text_glyph_colors(
    graphics_entity: Entity,
    colors: Vec<Color>,
) -> error::Result<()> {
    graphics_record_command(graphics_entity, DrawCommand::TextGlyphColors(colors))
}

/// Snapshot a graphics entity's text state and the shared `TextContext`.
fn text_query_state(
    app: &App,
    graphics_entity: Entity,
    max_w: Option<f32>,
    max_h: Option<f32>,
) -> error::Result<(render::primitive::text::OwnedTextParams, text::font::TextContext)> {
    let state = app
        .world()
        .get::<render::RenderState>(graphics_entity)
        .ok_or(error::ProcessingError::GraphicsNotFound)?;
    let params = render::primitive::text::OwnedTextParams::from_render_state(state, max_w, max_h);
    let text_cx = app.world().resource::<text::font::TextContext>().clone();
    Ok((params, text_cx))
}

pub fn graphics_text_to_paths(
    graphics_entity: Entity,
    content: &str,
    x: f32,
    y: f32,
) -> error::Result<Vec<Vec<render::primitive::text::PathCommand>>> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, None, None)?;
        Ok(render::primitive::text::text_to_paths(
            content,
            x,
            y,
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_to_contours(
    graphics_entity: Entity,
    content: &str,
    x: f32,
    y: f32,
) -> error::Result<Vec<Vec<render::primitive::text::PathCommand>>> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, None, None)?;
        Ok(render::primitive::text::text_to_contours(
            content,
            x,
            y,
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_to_points(
    graphics_entity: Entity,
    content: &str,
    x: f32,
    y: f32,
    sample_factor: Option<f32>,
) -> error::Result<Vec<[f32; 2]>> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, None, None)?;
        Ok(render::primitive::text::text_to_points(
            content,
            x,
            y,
            sample_factor.unwrap_or(render::primitive::text::DEFAULT_SAMPLE_FACTOR),
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_to_model(
    graphics_entity: Entity,
    content: &str,
    x: f32,
    y: f32,
    depth: f32,
) -> error::Result<Mesh> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, None, None)?;
        Ok(render::primitive::text::text_to_model(
            content,
            x,
            y,
            depth,
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_width(graphics_entity: Entity, content: &str) -> error::Result<f32> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, None, None)?;
        Ok(render::primitive::text::text_width(
            content,
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_ascent(graphics_entity: Entity) -> error::Result<f32> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, None, None)?;
        Ok(render::primitive::text::text_ascent(
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_descent(graphics_entity: Entity) -> error::Result<f32> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, None, None)?;
        Ok(render::primitive::text::text_descent(
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_bounds(
    graphics_entity: Entity,
    content: &str,
    x: f32,
    y: f32,
    max_w: Option<f32>,
    max_h: Option<f32>,
) -> error::Result<[f32; 4]> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, max_w, max_h)?;
        Ok(render::primitive::text::text_bounds(
            content,
            x,
            y,
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_line_count(
    graphics_entity: Entity,
    content: &str,
) -> error::Result<usize> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, None, None)?;
        Ok(render::primitive::text::text_line_count(
            content,
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_lines(
    graphics_entity: Entity,
    content: &str,
    x: f32,
    y: f32,
    max_w: Option<f32>,
    max_h: Option<f32>,
) -> error::Result<Vec<render::primitive::text::TextLineInfo>> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, max_w, max_h)?;
        Ok(render::primitive::text::text_lines(
            content,
            x,
            y,
            &params.as_params(),
            &text_cx,
        ))
    })
}

pub fn graphics_text_glyph_rects(
    graphics_entity: Entity,
    content: &str,
    x: f32,
    y: f32,
    max_w: Option<f32>,
    max_h: Option<f32>,
) -> error::Result<Vec<render::primitive::text::TextGlyphInfo>> {
    app_mut(|app| {
        let (params, text_cx) = text_query_state(app, graphics_entity, max_w, max_h)?;
        Ok(render::primitive::text::text_glyph_rects(
            content,
            x,
            y,
            &params.as_params(),
            &text_cx,
        ))
    })
}
