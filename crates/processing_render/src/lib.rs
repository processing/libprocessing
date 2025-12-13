pub mod error;
mod graphics;
pub mod image;
pub mod render;
mod surface;

use std::{cell::RefCell, num::NonZero, path::PathBuf, sync::OnceLock};

#[cfg(not(target_arch = "wasm32"))]
use bevy::log::tracing_subscriber;
use bevy::{
    app::{App, AppExit},
    asset::AssetEventSystems,
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
};
use render::{activate_cameras, clear_transient_meshes, flush_draw_commands};
use tracing::debug;

use crate::{
    graphics::GraphicsPlugin, image::ImagePlugin, render::command::DrawCommand,
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
        surface::create_surface_macos(app.world_mut(), window_handle, width, height, scale_factor)
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
        surface::create_surface_windows(app.world_mut(), window_handle, width, height, scale_factor)
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
        surface::create_surface_wayland(
            app.world_mut(),
            window_handle,
            display_handle,
            width,
            height,
            scale_factor,
        )
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
        surface::create_surface_x11(
            app.world_mut(),
            window_handle,
            display_handle,
            width,
            height,
            scale_factor,
        )
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
        surface::create_surface_web(app.world_mut(), window_handle, width, height, scale_factor)
    })
}

pub fn surface_create_offscreen(
    width: u32,
    height: u32,
    scale_factor: f32,
    texture_format: TextureFormat,
) -> error::Result<Entity> {
    app_mut(|app| {
        surface::create_offscreen(app.world_mut(), width, height, scale_factor, texture_format)
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
    app_mut(|app| surface::destroy(app.world_mut(), graphics_entity))
}

/// Update window size when resized.
pub fn surface_resize(graphics_entity: Entity, width: u32, height: u32) -> error::Result<()> {
    app_mut(|app| surface::resize(app.world_mut(), graphics_entity, width, height))
}

fn create_app() -> App {
    let mut app = App::new();

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
    app.add_plugins((ImagePlugin, GraphicsPlugin, SurfacePlugin));
    app.add_systems(First, (clear_transient_meshes, activate_cameras))
        .add_systems(Update, flush_draw_commands.before(AssetEventSystems));

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
pub fn init() -> error::Result<()> {
    setup_tracing()?;
    if is_already_init()? {
        return Ok(());
    }

    let mut app = create_app();
    app.finish();
    app.cleanup();
    set_app(app);

    Ok(())
}

/// Initialize the app asynchronously
#[cfg(target_arch = "wasm32")]
pub async fn init() -> error::Result<()> {
    use bevy::app::PluginsState;

    setup_tracing()?;
    if is_already_init()? {
        return Ok(());
    }

    let mut app = create_app();

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
    app_mut(|app| graphics::create(app.world_mut(), surface_entity, width, height))
}

/// Begin a new draw pass for the graphics surface.
pub fn graphics_begin_draw(_graphics_entity: Entity) -> error::Result<()> {
    app_mut(|app| graphics::begin_draw(app, _graphics_entity))
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
    app_mut(|app| graphics::destroy(app.world_mut(), graphics_entity))
}

/// Read back pixel data from the graphics surface.
pub fn graphics_readback(graphics_entity: Entity) -> error::Result<Vec<LinearRgba>> {
    app_mut(|app| {
        graphics::flush(app, graphics_entity)?;
        graphics::readback(app.world_mut(), graphics_entity)
    })
}

/// Update the graphics surface with new pixel data.
pub fn graphics_update(graphics_entity: Entity, pixels: &[LinearRgba]) -> error::Result<()> {
    app_mut(|app| graphics::update(app.world_mut(), graphics_entity, pixels))
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
        graphics::update_region(
            app.world_mut(),
            graphics_entity,
            x,
            y,
            width,
            height,
            pixels,
        )
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
    app_mut(|app| graphics::record_command(app.world_mut(), graphics_entity, cmd))
}

/// Create a new image with given size and data.
pub fn image_create(
    size: Extent3d,
    data: Vec<u8>,
    texture_format: TextureFormat,
) -> error::Result<Entity> {
    app_mut(|app| Ok(image::create(app.world_mut(), size, data, texture_format)))
}

/// Load an image from disk.
#[cfg(not(target_arch = "wasm32"))]
pub fn image_load(path: &str) -> error::Result<Entity> {
    let path = PathBuf::from(path);
    app_mut(|app| image::load(app.world_mut(), path))
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

    app_mut(|app| image::from_handle(app.world_mut(), handle))
}

/// Resize an existing image to new size.
pub fn image_resize(entity: Entity, new_size: Extent3d) -> error::Result<()> {
    app_mut(|app| image::resize(app.world_mut(), entity, new_size))
}

/// Read back image data from GPU to CPU.
pub fn image_readback(entity: Entity) -> error::Result<Vec<LinearRgba>> {
    app_mut(|app| image::readback(app.world_mut(), entity))
}

/// Update an existing image with new pixel data.
pub fn image_update(entity: Entity, pixels: &[LinearRgba]) -> error::Result<()> {
    app_mut(|app| image::update(app.world_mut(), entity, pixels))
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
    app_mut(|app| image::update_region(app.world_mut(), entity, x, y, width, height, pixels))
}

/// Destroy an existing image and free its resources.
pub fn image_destroy(entity: Entity) -> error::Result<()> {
    app_mut(|app| image::destroy(app.world_mut(), entity))
}
