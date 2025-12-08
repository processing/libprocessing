pub mod error;
pub mod image;
pub mod render;

use std::{cell::RefCell, num::NonZero, path::PathBuf, ptr::NonNull, sync::OnceLock};

#[cfg(any(target_os = "linux", target_arch = "wasm32"))]
use std::ffi::c_void;

use bevy::{
    app::{App, AppExit},
    asset::AssetEventSystems,
    camera::{CameraOutputMode, CameraProjection, RenderTarget, visibility::RenderLayers},
    log::tracing_subscriber,
    math::Vec3A,
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
    window::{RawHandleWrapper, Window, WindowRef, WindowResolution, WindowWrapper},
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};
use render::{activate_cameras, clear_transient_meshes, flush_draw_commands};
use tracing::debug;

use crate::{
    error::Result,
    render::command::{CommandBuffer, DrawCommand},
};

static IS_INIT: OnceLock<()> = OnceLock::new();

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

#[derive(Resource, Default)]
struct WindowCount(u32);

#[derive(Component)]
pub struct Flush;

#[derive(Component)]
pub struct SurfaceSize(u32, u32);

/// Custom orthographic projection for Processing's coordinate system.
/// Origin at top-left, Y-axis down, in pixel units (aka screen space).
#[derive(Debug, Clone, Reflect)]
#[reflect(Default)]
pub struct ProcessingProjection {
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for ProcessingProjection {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            near: 0.0,
            far: 1000.0,
        }
    }
}

impl CameraProjection for ProcessingProjection {
    fn get_clip_from_view(&self) -> Mat4 {
        Mat4::orthographic_rh(
            0.0,
            self.width,
            self.height, // bottom = height
            0.0,         // top = 0
            self.near,
            self.far,
        )
    }

    fn get_clip_from_view_for_sub(&self, _sub_view: &bevy::camera::SubCameraView) -> Mat4 {
        // TODO: implement sub-view support if needed (probably not)
        self.get_clip_from_view()
    }

    fn update(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    fn far(&self) -> f32 {
        self.far
    }

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        // order: bottom-right, top-right, top-left, bottom-left for near, then far
        let near_center = Vec3A::new(self.width / 2.0, self.height / 2.0, z_near);
        let far_center = Vec3A::new(self.width / 2.0, self.height / 2.0, z_far);

        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        [
            // near plane
            near_center + Vec3A::new(half_width, half_height, 0.0), // bottom-right
            near_center + Vec3A::new(half_width, -half_height, 0.0), // top-right
            near_center + Vec3A::new(-half_width, -half_height, 0.0), // top-left
            near_center + Vec3A::new(-half_width, half_height, 0.0), // bottom-left
            // far plane
            far_center + Vec3A::new(half_width, half_height, 0.0), // bottom-right
            far_center + Vec3A::new(half_width, -half_height, 0.0), // top-right
            far_center + Vec3A::new(-half_width, -half_height, 0.0), // top-left
            far_center + Vec3A::new(-half_width, half_height, 0.0), // bottom-left
        ]
    }
}

fn app_mut<T>(cb: impl FnOnce(&mut App) -> Result<T>) -> Result<T> {
    let res = APP.with(|app_cell| {
        let mut app_borrow = app_cell.borrow_mut();
        let app = app_borrow
            .as_mut()
            .ok_or(error::ProcessingError::AppAccess)?;
        cb(app)
    })?;
    Ok(res)
}

struct GlfwWindow {
    window_handle: RawWindowHandle,
    display_handle: RawDisplayHandle,
}

// SAFETY:
//  - RawWindowHandle and RawDisplayHandle are just pointers
//  - The actual window is managed by Java and outlives this struct
//  - GLFW is thread-safe-ish, see https://www.glfw.org/faq#29---is-glfw-thread-safe
//
// Note: we enforce that all calls to init/update/exit happen on the main thread, so
// there should be no concurrent access to the window from multiple threads anyway.
unsafe impl Send for GlfwWindow {}
unsafe impl Sync for GlfwWindow {}

impl HasWindowHandle for GlfwWindow {
    fn window_handle(&self) -> core::result::Result<WindowHandle<'_>, HandleError> {
        // SAFETY:
        //  - Handles passed from Java are valid
        Ok(unsafe { WindowHandle::borrow_raw(self.window_handle) })
    }
}

impl HasDisplayHandle for GlfwWindow {
    fn display_handle(&self) -> core::result::Result<DisplayHandle<'_>, HandleError> {
        // SAFETY:
        //  - Handles passed from Java are valid
        Ok(unsafe { DisplayHandle::borrow_raw(self.display_handle) })
    }
}

/// Create a WebGPU surface from a native window handle.
///
/// Currently, this just creates a bevy window with the given parameters and
/// stores the raw window handle for later use by the renderer, which will
/// actually create the surface.
pub fn surface_create(
    window_handle: u64,
    display_handle: u64,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> Result<Entity> {
    #[cfg(target_os = "macos")]
    let (raw_window_handle, raw_display_handle) = {
        use raw_window_handle::{AppKitDisplayHandle, AppKitWindowHandle};

        // GLFW gives us NSWindow*, but AppKitWindowHandle needs NSView*
        // so we have to do some objc magic to grab the right pointer
        let ns_view_ptr = {
            use objc2::rc::Retained;
            use objc2_app_kit::{NSView, NSWindow};

            // SAFETY:
            //  - window_handle is a valid NSWindow pointer from the GLFW window
            let ns_window = window_handle as *mut NSWindow;
            if ns_window.is_null() {
                return Err(error::ProcessingError::InvalidWindowHandle);
            }

            // SAFETY:
            // - The contentView is owned by NSWindow and remains valid as long as the window exists
            let ns_window_ref = unsafe { &*ns_window };
            let content_view: Option<Retained<NSView>> = ns_window_ref.contentView();

            match content_view {
                Some(view) => Retained::as_ptr(&view) as *mut std::ffi::c_void,
                None => {
                    return Err(error::ProcessingError::InvalidWindowHandle);
                }
            }
        };

        let window = AppKitWindowHandle::new(NonNull::new(ns_view_ptr).unwrap());
        let display = AppKitDisplayHandle::new();
        (
            RawWindowHandle::AppKit(window),
            RawDisplayHandle::AppKit(display),
        )
    };

    #[cfg(target_os = "windows")]
    let (raw_window_handle, raw_display_handle) = {
        use std::num::NonZeroIsize;

        use raw_window_handle::{Win32WindowHandle, WindowsDisplayHandle};
        use windows::Win32::{Foundation::HINSTANCE, System::LibraryLoader::GetModuleHandleW};

        if window_handle == 0 {
            return Err(error::ProcessingError::InvalidWindowHandle);
        }

        // HWND is isize, so cast it
        let hwnd_isize = window_handle as isize;
        let hwnd_nonzero = match NonZeroIsize::new(hwnd_isize) {
            Some(nz) => nz,
            None => return Err(error::ProcessingError::InvalidWindowHandle),
        };

        let mut window = Win32WindowHandle::new(hwnd_nonzero);

        // VK_KHR_win32_surface requires hinstance *and* hwnd
        // SAFETY: GetModuleHandleW(NULL) is safe
        let hinstance = unsafe { GetModuleHandleW(None) }
            .map_err(|_| error::ProcessingError::InvalidWindowHandle)?;

        let hinstance_nonzero = NonZeroIsize::new(hinstance.0 as isize)
            .ok_or(error::ProcessingError::InvalidWindowHandle)?;
        window.hinstance = Some(hinstance_nonzero);

        let display = WindowsDisplayHandle::new();

        (
            RawWindowHandle::Win32(window),
            RawDisplayHandle::Windows(display),
        )
    };

    #[cfg(target_os = "linux")]
    let (raw_window_handle, raw_display_handle) = {
        use raw_window_handle::{WaylandDisplayHandle, WaylandWindowHandle};

        if window_handle == 0 {
            return Err(error::ProcessingError::HandleError(
                HandleError::Unavailable,
            ));
        }
        let window_handle_ptr = NonNull::new(window_handle as *mut c_void).unwrap();
        let window = WaylandWindowHandle::new(window_handle_ptr);

        if display_handle == 0 {
            return Err(error::ProcessingError::HandleError(
                HandleError::Unavailable,
            ));
        }
        let display_handle_ptr = NonNull::new(display_handle as *mut c_void).unwrap();
        let display = WaylandDisplayHandle::new(display_handle_ptr);

        (
            RawWindowHandle::Wayland(window),
            RawDisplayHandle::Wayland(display),
        )
    };

    #[cfg(target_arch = "wasm32")]
    let (raw_window_handle, raw_display_handle) = {
        use raw_window_handle::{WebCanvasWindowHandle, WebDisplayHandle};

        // window_handle is a pointer to HtmlCanvasElement DOM obj
        let canvas_ptr = window_handle as *mut c_void;
        let canvas = NonNull::new(canvas_ptr).ok_or(error::ProcessingError::InvalidWindowHandle)?;

        let window = WebCanvasWindowHandle::new(canvas);
        let display = WebDisplayHandle::new();
        (
            RawWindowHandle::WebCanvas(window),
            RawDisplayHandle::Web(display),
        )
    };

    let glfw_window = GlfwWindow {
        window_handle: raw_window_handle,
        display_handle: raw_display_handle,
    };

    let window_wrapper = WindowWrapper::new(glfw_window);
    let handle_wrapper = RawHandleWrapper::new(&window_wrapper)?;

    let entity_id = app_mut(|app| {
        let mut window_count = app.world_mut().resource_mut::<WindowCount>();
        let count = window_count.0;
        window_count.0 += 1;
        let render_layer = RenderLayers::none().with(count as usize);

        let mut window = app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(width, height)
                    .with_scale_factor_override(scale_factor),
                ..default()
            },
            handle_wrapper,
            CommandBuffer::default(),
            // this doesn't do anything but makes it easier to fetch the render layer for
            // meshes to be drawn to this window
            render_layer.clone(),
            SurfaceSize(width, height),
        ));

        let window_entity = window.id();
        window.with_children(|parent| {
            // processing has a different coordinate system for 2d rendering:
            // - origin at top-left
            // - x increases to the right, y increases downward
            // - coordinate units are in screen pixels
            parent.spawn((
                Camera3d::default(),
                Camera {
                    target: RenderTarget::Window(WindowRef::Entity(window_entity)),
                    ..default()
                },
                Projection::custom(ProcessingProjection {
                    width: width as f32,
                    height: height as f32,
                    near: 0.0,
                    far: 1000.0,
                }),
                Transform::from_xyz(0.0, 0.0, 999.9),
                render_layer,
            ));
        });

        Ok(window_entity)
    })?;

    Ok(entity_id)
}

/// Create a WebGPU surface from a canvas element ID
#[cfg(target_arch = "wasm32")]
pub fn surface_create_from_canvas(canvas_id: &str, width: u32, height: u32) -> Result<Entity> {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlCanvasElement;

    // find the canvas elelment
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

    surface_create(canvas_ptr, 0, width, height, scale_factor)
}

pub fn surface_destroy(window_entity: Entity) -> Result<()> {
    app_mut(|app| {
        if app.world_mut().get::<Window>(window_entity).is_some() {
            app.world_mut().despawn(window_entity);
            let mut window_count = app.world_mut().resource_mut::<WindowCount>();
            window_count.0 = window_count.0.saturating_sub(1);
        }
        Ok(())
    })
}

/// Update window size when resized.
pub fn surface_resize(window_entity: Entity, width: u32, height: u32) -> Result<()> {
    app_mut(|app| {
        if let Some(mut window) = app.world_mut().get_mut::<Window>(window_entity) {
            window.resolution.set_physical_resolution(width, height);
        } else {
            return Err(error::ProcessingError::WindowNotFound);
        };
        app.world_mut()
            .entity_mut(window_entity)
            .insert(SurfaceSize(width, height));
        Ok(())
    })
}

fn create_app() -> App {
    let mut app = App::new();

    #[cfg(not(target_arch = "wasm32"))]
    let plugins = DefaultPlugins
        .build()
        .disable::<bevy::log::LogPlugin>()
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>()
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        });

    #[cfg(target_arch = "wasm32")]
    let plugins = DefaultPlugins
        .build()
        .disable::<bevy::log::LogPlugin>()
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::audio::AudioPlugin>()
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        });

    app.add_plugins(plugins);
    app.init_resource::<WindowCount>();
    app.add_systems(First, (clear_transient_meshes, activate_cameras))
        .add_systems(Update, flush_draw_commands.before(AssetEventSystems));

    app
}

fn is_already_init() -> Result<bool> {
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
pub fn init() -> Result<()> {
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
pub async fn init() -> Result<()> {
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

macro_rules! camera_mut {
    ($app:expr, $window_entity:expr) => {
        $app.world_mut()
            .query::<(&mut Camera, &ChildOf)>()
            .iter_mut(&mut $app.world_mut())
            .filter_map(|(camera, parent)| {
                if parent.parent() == $window_entity {
                    Some(camera)
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| error::ProcessingError::WindowNotFound)?
    };
}

macro_rules! window_mut {
    ($app:expr, $window_entity:expr) => {
        $app.world_mut()
            .get_entity_mut($window_entity)
            .map_err(|_| error::ProcessingError::WindowNotFound)?
    };
}

pub fn begin_draw(_window_entity: Entity) -> Result<()> {
    app_mut(|_app| Ok(()))
}

pub fn flush(window_entity: Entity) -> Result<()> {
    app_mut(|app| {
        window_mut!(app, window_entity).insert(Flush);
        app.update();
        window_mut!(app, window_entity).remove::<Flush>();

        // ensure that the intermediate texture is not cleared
        camera_mut!(app, window_entity).clear_color = ClearColorConfig::None;
        Ok(())
    })
}

pub fn end_draw(window_entity: Entity) -> Result<()> {
    // since we are ending the draw, set the camera to write to the output render target
    app_mut(|app| {
        camera_mut!(app, window_entity).output_mode = CameraOutputMode::Write {
            blend_state: None,
            clear_color: ClearColorConfig::Default,
        };
        Ok(())
    })?;
    // flush any remaining draw commands, this ensures that the frame is presented even if there
    // is no remaining draw commands
    flush(window_entity)?;
    // reset to skipping output for the next frame
    app_mut(|app| {
        camera_mut!(app, window_entity).output_mode = CameraOutputMode::Skip;
        Ok(())
    })
}

pub fn exit(exit_code: u8) -> Result<()> {
    app_mut(|app| {
        app.world_mut().write_message(match exit_code {
            0 => AppExit::Success,
            _ => AppExit::Error(NonZero::new(exit_code).unwrap()),
        });

        // one final update to process the exit message
        app.update();
        Ok(())
    })?;

    // we need to drop the app in a deterministic manner to ensure resourcse are cleaned up
    // otherwise we'll get wgpu graphics backend errors on exit
    APP.with(|app_cell| {
        let app = app_cell.borrow_mut().take();
        drop(app);
    });

    Ok(())
}

pub fn background_color(window_entity: Entity, color: Color) -> Result<()> {
    app_mut(|app| {
        let mut camera_query = app.world_mut().query::<(&mut Camera, &ChildOf)>();
        for (mut camera, parent) in camera_query.iter_mut(app.world_mut()) {
            if parent.parent() == window_entity {
                camera.clear_color = ClearColorConfig::Custom(color);
            }
        }
        Ok(())
    })
}

fn setup_tracing() -> Result<()> {
    // TODO: figure out wasm compatible tracing subscriber
    #[cfg(not(target_arch = "wasm32"))]
    {
        let subscriber = tracing_subscriber::FmtSubscriber::new();
        tracing::subscriber::set_global_default(subscriber)?;
    }
    Ok(())
}

/// Record a drawing command for a window
pub fn record_command(window_entity: Entity, cmd: DrawCommand) -> Result<()> {
    app_mut(|app| {
        let mut entity_mut = app.world_mut().entity_mut(window_entity);
        if let Some(mut buffer) = entity_mut.get_mut::<CommandBuffer>() {
            buffer.push(cmd);
        }

        Ok(())
    })
}

/// Create a new image with given size and data.
pub fn image_create(
    size: Extent3d,
    data: Vec<u8>,
    texture_format: TextureFormat,
) -> Result<Entity> {
    app_mut(|app| Ok(image::create(app.world_mut(), size, data, texture_format)))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn image_load(path: &str) -> Result<Entity> {
    let path = PathBuf::from(path);
    app_mut(|app| image::load(app.world_mut(), path))
}

#[cfg(target_arch = "wasm32")]
pub async fn image_load(path: &str) -> Result<Entity> {
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
pub fn image_resize(entity: Entity, new_size: Extent3d) -> Result<()> {
    app_mut(|app| image::resize(app.world_mut(), entity, new_size))
}

/// Read back image data from GPU to CPU.
pub fn image_load_pixels(entity: Entity) -> Result<Vec<LinearRgba>> {
    app_mut(|app| image::load_pixels(app.world_mut(), entity))
}

/// Destroy an existing image and free its resources.
pub fn image_destroy(entity: Entity) -> Result<()> {
    app_mut(|app| image::destroy(app.world_mut(), entity))
}
