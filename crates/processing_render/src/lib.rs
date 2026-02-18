pub mod config;
pub mod error;
pub mod geometry;
mod graphics;
pub mod image;
pub mod light;
pub mod material;
pub mod render;
pub mod sketch;
mod surface;
pub mod transform;

use std::{cell::RefCell, num::NonZero, path::PathBuf, sync::OnceLock};

use config::*;

#[cfg(not(target_arch = "wasm32"))]
use bevy::log::tracing_subscriber;
use bevy::{
    app::{App, AppExit},
    asset::{AssetEventSystems, io::AssetSourceBuilder},
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
};
use render::material::add_standard_materials;
use render::{activate_cameras, clear_transient_meshes, flush_draw_commands};
use tracing::debug;

use crate::geometry::{AttributeFormat, AttributeValue};
use crate::graphics::flush;
use crate::{
    graphics::GraphicsPlugin, image::ImagePlugin, light::LightPlugin, render::command::DrawCommand,
    surface::SurfacePlugin,
};

static IS_INIT: OnceLock<()> = OnceLock::new();

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

#[derive(Component)]
pub struct Flush;

fn app_mut<T>(cb: impl FnOnce(&mut App) -> error::Result<T>) -> error::Result<T> {
    let res = APP.with(|app_cell| {
        let mut app_borrow = app_cell.borrow_mut();
        let app = app_borrow
            .as_mut()
            .ok_or(error::ProcessingError::AppAccess)?;
        cb(app)
    })?;
    Ok(res)
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

fn create_app(config: Config) -> App {
    let mut app = App::new();

    app.insert_resource(config.clone());

    if let Some(asset_path) = config.get(ConfigKey::AssetRootPath) {
        app.register_asset_source(
            "assets_directory",
            AssetSourceBuilder::platform_default(asset_path, None),
        );
    }

    if let Some(sketch_path) = config.get(ConfigKey::SketchRootPath) {
        app.register_asset_source(
            "sketch_directory",
            AssetSourceBuilder::platform_default(sketch_path, None),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    let plugins = DefaultPlugins
        .build()
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::log::LogPlugin>()
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        });

    #[cfg(target_arch = "wasm32")]
    let plugins = DefaultPlugins
        .build()
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::log::LogPlugin>()
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        });

    app.add_plugins(plugins);

    if config.get(ConfigKey::SketchRootPath).is_some() {
        info!("Adding plugin");
        app.add_plugins(sketch::LivecodePlugin);
    }

    app.add_plugins((
        ImagePlugin,
        GraphicsPlugin,
        SurfacePlugin,
        geometry::GeometryPlugin,
        LightPlugin,
        material::MaterialPlugin,
    ));
    app.add_systems(First, (clear_transient_meshes, activate_cameras))
        .add_systems(
            Update,
            (flush_draw_commands, add_standard_materials)
                .chain()
                .before(AssetEventSystems),
        );

    app
}

fn is_already_init() -> error::Result<bool> {
    let is_init = IS_INIT.get().is_some();
    let thread_has_app = APP.with(|app_cell| app_cell.borrow().is_some());
    if is_init && !thread_has_app {
        return Err(error::ProcessingError::AppAccess);
    }
    if is_init && thread_has_app {
        debug!("App already initialized");
        return Ok(true);
    }
    Ok(false)
}

fn set_app(app: App) {
    APP.with(|app_cell| {
        IS_INIT.get_or_init(|| ());
        *app_cell.borrow_mut() = Some(app);
    });
}

/// Initialize the app, if not already initialized. Must be called from the main thread and cannot
/// be called concurrently from multiple threads.
#[cfg(not(target_arch = "wasm32"))]
pub fn init(config: Config) -> error::Result<()> {
    setup_tracing()?;
    if is_already_init()? {
        return Ok(());
    }

    let mut app = create_app(config);
    app.finish();
    app.cleanup();
    set_app(app);

    Ok(())
}

/// Initialize the app asynchronously
#[cfg(target_arch = "wasm32")]
pub async fn init(config: Config) -> error::Result<()> {
    use bevy::app::PluginsState;

    setup_tracing()?;
    if is_already_init()? {
        return Ok(());
    }

    let mut app = create_app(config);

    // we need to avoid blocking the main thread while waiting for plugins to initialize
    while app.plugins_state() == PluginsState::Adding {
        // yield to event loop
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
                .unwrap();
        }))
        .await
        .unwrap();
    }

    app.finish();
    app.cleanup();
    set_app(app);

    Ok(())
}

/// Create a new graphics surface for rendering.
pub fn graphics_create(surface_entity: Entity, width: u32, height: u32) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(graphics::create, (width, height, surface_entity))
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

/// Read back pixel data from the graphics surface.
pub fn graphics_readback(graphics_entity: Entity) -> error::Result<Vec<LinearRgba>> {
    app_mut(|app| {
        graphics::flush(app, graphics_entity)?;
        app.world_mut()
            .run_system_cached_with(graphics::readback, graphics_entity)
            .unwrap()
    })
}

/// Update the graphics surface with new pixel data.
pub fn graphics_update(graphics_entity: Entity, pixels: &[LinearRgba]) -> error::Result<()> {
    app_mut(|app| {
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
        let world = app.world_mut();
        let (data, px_size) =
            graphics::prepare_update_region(world, graphics_entity, width, height, pixels)?;
        world
            .run_system_cached_with(
                graphics::update_region_write,
                (graphics_entity, x, y, width, height, data, px_size),
            )
            .unwrap()
    })
}

pub fn exit(exit_code: u8) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut().write_message(match exit_code {
            0 => AppExit::Success,
            _ => AppExit::Error(NonZero::new(exit_code).unwrap()),
        });

        // one final update to process the exit message
        app.update();
        Ok(())
    })?;

    // we need to drop the app in a deterministic manner to ensure resources are cleaned up
    // otherwise we'll get wgpu graphics backend errors on exit
    APP.with(|app_cell| {
        let app = app_cell.borrow_mut().take();
        drop(app);
    });

    Ok(())
}

fn setup_tracing() -> error::Result<()> {
    // TODO: figure out wasm compatible tracing subscriber
    #[cfg(not(target_arch = "wasm32"))]
    {
        let subscriber = tracing_subscriber::FmtSubscriber::new();
        tracing::subscriber::set_global_default(subscriber)?;
    }
    Ok(())
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

pub fn graphics_perspective(
    graphics_entity: Entity,
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
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

pub fn transform_set_position(entity: Entity, x: f32, y: f32, z: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::set_position, (entity, x, y, z))
            .unwrap()
    })
}

pub fn transform_translate(entity: Entity, x: f32, y: f32, z: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::translate, (entity, x, y, z))
            .unwrap()
    })
}

pub fn transform_set_rotation(entity: Entity, x: f32, y: f32, z: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::set_rotation, (entity, x, y, z))
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

pub fn transform_rotate_axis(
    entity: Entity,
    angle: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(
                transform::rotate_axis,
                (entity, angle, axis_x, axis_y, axis_z),
            )
            .unwrap()
    })
}

pub fn transform_set_scale(entity: Entity, x: f32, y: f32, z: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::set_scale, (entity, x, y, z))
            .unwrap()
    })
}

pub fn transform_scale(entity: Entity, x: f32, y: f32, z: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::scale, (entity, x, y, z))
            .unwrap()
    })
}

pub fn transform_look_at(
    entity: Entity,
    target_x: f32,
    target_y: f32,
    target_z: f32,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(transform::look_at, (entity, target_x, target_y, target_z))
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
        app.world_mut()
            .run_system_cached_with(image::readback, entity)
            .unwrap()
    })
}

/// Update an existing image with new pixel data.
pub fn image_update(entity: Entity, pixels: &[LinearRgba]) -> error::Result<()> {
    app_mut(|app| {
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
                (entity, 0, 0, size.width, size.height, data, px_size),
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
        let world = app.world_mut();
        let (data, px_size) = image::prepare_update_region(world, entity, width, height, pixels)?;
        world
            .run_system_cached_with(
                image::update_region_write,
                (entity, x, y, width, height, data, px_size),
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

pub fn geometry_normal(entity: Entity, nx: f32, ny: f32, nz: f32) -> error::Result<()> {
    app_mut(|app| geometry::normal(app.world_mut(), entity, nx, ny, nz))
}

pub fn geometry_color(entity: Entity, r: f32, g: f32, b: f32, a: f32) -> error::Result<()> {
    app_mut(|app| geometry::color(app.world_mut(), entity, r, g, b, a))
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

pub fn geometry_vertex(entity: Entity, x: f32, y: f32, z: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::vertex, (entity, x, y, z))
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

pub fn geometry_set_vertex(
    entity: Entity,
    index: u32,
    x: f32,
    y: f32,
    z: f32,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::set_vertex, (entity, index, x, y, z))
            .unwrap()
    })
}

pub fn geometry_set_normal(
    entity: Entity,
    index: u32,
    nx: f32,
    ny: f32,
    nz: f32,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::set_normal, (entity, index, nx, ny, nz))
            .unwrap()
    })
}

pub fn geometry_set_color(
    entity: Entity,
    index: u32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::set_color, (entity, index, r, g, b, a))
            .unwrap()
    })
}

pub fn geometry_set_uv(entity: Entity, index: u32, u: f32, v: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(geometry::set_uv, (entity, index, u, v))
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
