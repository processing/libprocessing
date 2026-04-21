#![allow(clippy::module_inception)]

pub mod camera;
pub mod color;
pub mod geometry;
pub mod gltf;
pub mod graphics;
pub mod image;
pub mod light;
pub mod material;
pub mod monitor;
pub mod render;
pub mod sketch;
pub(crate) mod surface;
pub mod time;
pub mod transform;

use std::path::PathBuf;

use bevy::{
    asset::{AssetEventSystems, io::AssetSourceBuilder},
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

        if let Some(asset_path) = config.get(ConfigKey::AssetRootPath) {
            app.register_asset_source(
                "assets_directory",
                AssetSourceBuilder::platform_default(asset_path, None),
            );
        }

        let has_sketch_file = config
            .get(ConfigKey::SketchFileName)
            .is_some_and(|f| !f.is_empty());
        if has_sketch_file && let Some(sketch_path) = config.get(ConfigKey::SketchRootPath) {
            app.register_asset_source(
                "sketch_directory",
                AssetSourceBuilder::platform_default(sketch_path, None),
            );
        }

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
            camera::OrbitCameraPlugin,
            bevy::camera_controller::free_camera::FreeCameraPlugin,
            bevy::camera_controller::pan_camera::PanCameraPlugin,
        ));

        app.add_systems(First, (clear_transient_meshes, activate_cameras))
            .add_systems(
                Update,
                (
                    flush_draw_commands,
                    add_processing_materials,
                    add_custom_materials,
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

/// Create a new graphics surface for rendering.
pub fn graphics_create(
    surface_entity: Entity,
    width: u32,
    height: u32,
    texture_format: TextureFormat,
) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                graphics::create,
                (width, height, surface_entity, texture_format),
            )
            .unwrap()
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

pub fn geometry_attribute_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::attribute::destroy, entity)
            .unwrap()?;
        Ok(())
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

/// Load a shader from a file path.
pub fn shader_load(path: &str) -> error::Result<Entity> {
    let path = std::path::PathBuf::from(path);
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

pub fn material_set(
    entity: Entity,
    name: impl Into<String>,
    value: material::MaterialValue,
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

pub fn frame_count() -> error::Result<u32> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached(time::frame_count)
            .unwrap())
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
