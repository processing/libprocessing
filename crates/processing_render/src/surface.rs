//! A "surface" in Processing is essentially a window or canvas where graphics are rendered. In
//! typical rendering backends, a surface corresponds to a native window, i.e. a swapchain. However,
//! processing allows for "offscreen" rendering via the `PSurfaceNone` type, which does not have a
//! native window associated with it. This module provides functionality to create and manage both
//! types of surfaces.
//!
//! In Bevy, we can consider a surface to be a [`RenderTarget`], which is either a window or a
//! texture.
//!
//! ## Platform-specific surface creation
//!
//! On Linux, both X11 and Wayland are supported via feature flags:
//! - `x11`: Enable X11 surface creation via `create_surface_x11`
//! - `wayland`: Enable Wayland surface creation via `create_surface_wayland`
//!
//! On other platforms, use the platform-specific functions:
//! - macOS: `create_surface_macos`
//! - Windows: `create_surface_windows`
//! - WebAssembly: `create_surface_web`

use bevy::{
    app::{App, Plugin},
    asset::Assets,
    ecs::query::QueryEntityError,
    math::{IRect, IVec2},
    prelude::{Commands, Component, Entity, In, Query, ResMut, Window, With, default},
    render::render_resource::{Extent3d, TextureFormat},
    window::{
        Monitor, RawHandleWrapper, WindowLevel, WindowMode, WindowPosition, WindowResolution,
        WindowWrapper,
    },
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};

use processing_core::error::{self, ProcessingError, Result};
#[cfg(not(target_os = "windows"))]
use std::ptr::NonNull;

use crate::image::Image;

#[derive(Component, Debug, Clone)]
pub struct Surface;

/// Window properties Bevy's [`Window`] doesn't model. Backends drain the pending fields
/// each tick.
#[derive(Component, Debug, Default, Clone)]
pub struct WindowControls {
    pub opacity: Option<f32>,
    pub pending_iconify: bool,
    pub pending_restore: bool,
    pub pending_maximize: bool,
    pub pending_focus: bool,
}

/// Usable region of a monitor (excluding taskbars / menu bars). Populated by the active
/// windowing backend.
#[derive(Component, Debug, Clone, Copy)]
pub struct MonitorWorkarea(pub IRect);

pub struct SurfacePlugin;

impl Plugin for SurfacePlugin {
    fn build(&self, _app: &mut App) {}
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

/// Helper to spawn a surface entity from raw handles.
fn spawn_surface(
    commands: &mut Commands,
    raw_window_handle: RawWindowHandle,
    raw_display_handle: RawDisplayHandle,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> Result<Entity> {
    let glfw_window = GlfwWindow {
        window_handle: raw_window_handle,
        display_handle: raw_display_handle,
    };

    let window_wrapper = WindowWrapper::new(glfw_window);
    let handle_wrapper = RawHandleWrapper::new(&window_wrapper)?;

    let physical_width = (width as f32 * scale_factor) as u32;
    let physical_height = (height as f32 * scale_factor) as u32;

    Ok(commands
        .spawn((
            Window {
                resolution: WindowResolution::new(physical_width, physical_height)
                    .with_scale_factor_override(scale_factor),
                ..default()
            },
            handle_wrapper,
            Surface,
            WindowControls::default(),
        ))
        .id())
}

/// Create a WebGPU surface from a macOS NSWindow handle.
///
/// # Arguments
/// * `window_handle` - A pointer to the NSWindow (from GLFW's `get_cocoa_window()`)
#[cfg(target_os = "macos")]
pub fn create_surface_macos(
    In((window_handle, width, height, scale_factor)): In<(u64, u32, u32, f32)>,
    mut commands: Commands,
) -> Result<Entity> {
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

    spawn_surface(
        &mut commands,
        RawWindowHandle::AppKit(window),
        RawDisplayHandle::AppKit(display),
        width,
        height,
        scale_factor,
    )
}

/// Create a WebGPU surface from a Windows HWND handle.
///
/// # Arguments
/// * `window_handle` - The HWND value (from GLFW's `get_win32_window()`)
#[cfg(target_os = "windows")]
pub fn create_surface_windows(
    In((window_handle, width, height, scale_factor)): In<(u64, u32, u32, f32)>,
    mut commands: Commands,
) -> Result<Entity> {
    use std::num::NonZeroIsize;

    use raw_window_handle::{Win32WindowHandle, WindowsDisplayHandle};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;

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

    spawn_surface(
        &mut commands,
        RawWindowHandle::Win32(window),
        RawDisplayHandle::Windows(display),
        width,
        height,
        scale_factor,
    )
}

/// Create a WebGPU surface from a Wayland window and display handle.
///
/// # Arguments
/// * `window_handle` - The wl_surface pointer (from GLFW's `get_wayland_window()`)
/// * `display_handle` - The wl_display pointer (from GLFW's `get_wayland_display()`)
#[cfg(all(target_os = "linux", feature = "wayland"))]
pub fn create_surface_wayland(
    In((window_handle, display_handle, width, height, scale_factor)): In<(u64, u64, u32, u32, f32)>,
    mut commands: Commands,
) -> Result<Entity> {
    use raw_window_handle::{WaylandDisplayHandle, WaylandWindowHandle};

    if window_handle == 0 {
        return Err(error::ProcessingError::HandleError(
            HandleError::Unavailable,
        ));
    }
    let window_handle_ptr = NonNull::new(window_handle as *mut std::ffi::c_void).unwrap();
    let window = WaylandWindowHandle::new(window_handle_ptr);

    if display_handle == 0 {
        return Err(error::ProcessingError::HandleError(
            HandleError::Unavailable,
        ));
    }
    let display_handle_ptr = NonNull::new(display_handle as *mut std::ffi::c_void).unwrap();
    let display = WaylandDisplayHandle::new(display_handle_ptr);

    spawn_surface(
        &mut commands,
        RawWindowHandle::Wayland(window),
        RawDisplayHandle::Wayland(display),
        width,
        height,
        scale_factor,
    )
}

/// Create a WebGPU surface from an X11 window and display handle.
///
/// # Arguments
/// * `window_handle` - The X11 Window ID (from GLFW's `get_x11_window()`)
/// * `display_handle` - The X11 Display pointer (from GLFW's `get_x11_display()`)
#[cfg(all(target_os = "linux", feature = "x11"))]
pub fn create_surface_x11(
    In((window_handle, display_handle, width, height, scale_factor)): In<(u64, u64, u32, u32, f32)>,
    mut commands: Commands,
) -> Result<Entity> {
    use raw_window_handle::{XlibDisplayHandle, XlibWindowHandle};

    if window_handle == 0 {
        return Err(error::ProcessingError::HandleError(
            HandleError::Unavailable,
        ));
    }
    // X11 Window is a u32/u64 ID, not a pointer
    let window = XlibWindowHandle::new(window_handle as std::ffi::c_ulong);

    if display_handle == 0 {
        return Err(error::ProcessingError::HandleError(
            HandleError::Unavailable,
        ));
    }
    let display_ptr = NonNull::new(display_handle as *mut c_void).unwrap();
    let display = XlibDisplayHandle::new(Some(display_ptr), 0); // screen 0

    spawn_surface(
        &mut commands,
        RawWindowHandle::Xlib(window),
        RawDisplayHandle::Xlib(display),
        width,
        height,
        scale_factor,
    )
}

/// Create a WebGPU surface from a web canvas element.
///
/// # Arguments
/// * `window_handle` - A pointer to the HtmlCanvasElement
#[cfg(target_arch = "wasm32")]
pub fn create_surface_web(
    In((window_handle, width, height, scale_factor)): In<(u64, u32, u32, f32)>,
    mut commands: Commands,
) -> Result<Entity> {
    use raw_window_handle::{WebCanvasWindowHandle, WebDisplayHandle};

    // For WASM, window_handle is a pointer to an HtmlCanvasElement
    if window_handle == 0 {
        return Err(error::ProcessingError::InvalidWindowHandle);
    }
    let canvas_ptr = NonNull::new(window_handle as *mut c_void).unwrap();
    let window = WebCanvasWindowHandle::new(canvas_ptr.cast());
    let display = WebDisplayHandle::new();

    spawn_surface(
        &mut commands,
        RawWindowHandle::WebCanvas(window),
        RawDisplayHandle::Web(display),
        width,
        height,
        scale_factor,
    )
}

pub fn prepare_offscreen(
    width: u32,
    height: u32,
    scale_factor: f32,
    texture_format: TextureFormat,
) -> Result<(Extent3d, Vec<u8>, TextureFormat)> {
    let size = Extent3d {
        width: (width as f32 * scale_factor) as u32,
        height: (height as f32 * scale_factor) as u32,
        depth_or_array_layers: 1,
    };
    let pixel_size = match texture_format {
        TextureFormat::R8Unorm => 1,
        TextureFormat::Rg8Unorm => 2,
        TextureFormat::Rgba8Unorm
        | TextureFormat::Rgba8UnormSrgb
        | TextureFormat::Bgra8Unorm
        | TextureFormat::Rgba16Float
        | TextureFormat::Rgba32Float => 4,
        _ => return Err(ProcessingError::UnsupportedTextureFormat),
    };

    let data = vec![0u8; (size.width * size.height * pixel_size) as usize];
    Ok((size, data, texture_format))
}

pub fn destroy(
    In(surface_entity): In<Entity>,
    mut commands: Commands,
    p_images: Query<&Image, With<Surface>>,
    mut images: ResMut<Assets<bevy::image::Image>>,
) -> Result<()> {
    match p_images.get(surface_entity) {
        Ok(p_image) => {
            images.remove(&p_image.handle);
            commands.entity(surface_entity).despawn();
            Ok(())
        }
        Err(QueryEntityError::QueryDoesNotMatch(..)) => {
            commands.entity(surface_entity).despawn();
            Ok(())
        }
        Err(_) => Err(ProcessingError::SurfaceNotFound),
    }
}

/// Update window size when resized. No-op on offscreen surfaces (no `Window` component).
pub fn resize(
    In((window_entity, width, height)): In<(Entity, u32, u32)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(window_entity) {
        let scale = window.resolution.scale_factor();
        let physical_w = (width as f32 * scale) as u32;
        let physical_h = (height as f32 * scale) as u32;
        window
            .resolution
            .set_physical_resolution(physical_w, physical_h);
    }
    Ok(())
}

pub fn set_pixel_density(
    In((window_entity, density)): In<(Entity, f32)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(window_entity) {
        let logical_w = window.resolution.width();
        let logical_h = window.resolution.height();
        window.resolution.set_scale_factor_override(Some(density));
        window
            .resolution
            .set_physical_resolution((logical_w * density) as u32, (logical_h * density) as u32);
        Ok(())
    } else {
        Err(error::ProcessingError::SurfaceNotFound)
    }
}

pub fn focused(In(entity): In<Entity>, query: Query<&Window>) -> bool {
    query.get(entity).map(|w| w.focused).unwrap_or(false)
}

pub fn scale_factor(In(entity): In<Entity>, query: Query<&Window>) -> f32 {
    query
        .get(entity)
        .map(|w| w.resolution.scale_factor())
        .unwrap_or(1.0)
}

pub fn physical_width(In(entity): In<Entity>, query: Query<&Window>) -> u32 {
    query
        .get(entity)
        .map(|w| w.resolution.physical_width())
        .unwrap_or(0)
}

pub fn physical_height(In(entity): In<Entity>, query: Query<&Window>) -> u32 {
    query
        .get(entity)
        .map(|w| w.resolution.physical_height())
        .unwrap_or(0)
}

// Windowed-surface ops are no-ops for entities without a [`Window`] component, matching
// PSurfaceNone's behaviour in Processing 4.

pub fn set_title(
    In((entity, title)): In<(Entity, String)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(entity) {
        window.title = title;
    }
    Ok(())
}

pub fn position(In(entity): In<Entity>, windows: Query<&Window>) -> IVec2 {
    match windows.get(entity).map(|w| w.position) {
        Ok(WindowPosition::At(p)) => p,
        _ => IVec2::ZERO,
    }
}

pub fn set_position(
    In((entity, x, y)): In<(Entity, i32, i32)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(entity) {
        window.position = WindowPosition::At(IVec2::new(x, y));
    }
    Ok(())
}

pub fn set_visible(
    In((entity, visible)): In<(Entity, bool)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(entity) {
        window.visible = visible;
    }
    Ok(())
}

pub fn set_resizable(
    In((entity, resizable)): In<(Entity, bool)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(entity) {
        window.resizable = resizable;
    }
    Ok(())
}

pub fn set_decorated(
    In((entity, decorated)): In<(Entity, bool)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(entity) {
        window.decorations = decorated;
    }
    Ok(())
}

pub fn set_window_level(
    In((entity, level)): In<(Entity, WindowLevel)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(entity) {
        window.window_level = level;
    }
    Ok(())
}

pub fn set_window_mode(
    In((entity, mode)): In<(Entity, WindowMode)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(entity) {
        window.mode = mode;
    }
    Ok(())
}

pub fn set_opacity(
    In((entity, opacity)): In<(Entity, f32)>,
    mut controls: Query<&mut WindowControls>,
) -> Result<()> {
    if let Ok(mut controls) = controls.get_mut(entity) {
        controls.opacity = Some(opacity.clamp(0.0, 1.0));
    }
    Ok(())
}

pub fn iconify(In(entity): In<Entity>, mut controls: Query<&mut WindowControls>) -> Result<()> {
    if let Ok(mut controls) = controls.get_mut(entity) {
        controls.pending_iconify = true;
    }
    Ok(())
}

pub fn restore(In(entity): In<Entity>, mut controls: Query<&mut WindowControls>) -> Result<()> {
    if let Ok(mut controls) = controls.get_mut(entity) {
        controls.pending_restore = true;
    }
    Ok(())
}

pub fn maximize(In(entity): In<Entity>, mut controls: Query<&mut WindowControls>) -> Result<()> {
    if let Ok(mut controls) = controls.get_mut(entity) {
        controls.pending_maximize = true;
    }
    Ok(())
}

pub fn focus(In(entity): In<Entity>, mut controls: Query<&mut WindowControls>) -> Result<()> {
    if let Ok(mut controls) = controls.get_mut(entity) {
        controls.pending_focus = true;
    }
    Ok(())
}

pub fn monitor_position(In(entity): In<Entity>, monitors: Query<&Monitor>) -> IVec2 {
    monitors
        .get(entity)
        .map(|m| m.physical_position)
        .unwrap_or(IVec2::ZERO)
}

pub fn monitor_workarea(
    In(entity): In<Entity>,
    monitors: Query<(&Monitor, Option<&MonitorWorkarea>)>,
) -> IRect {
    match monitors.get(entity) {
        Ok((_, Some(workarea))) => workarea.0,
        Ok((monitor, None)) => IRect::from_corners(
            monitor.physical_position,
            monitor.physical_position
                + IVec2::new(
                    monitor.physical_width as i32,
                    monitor.physical_height as i32,
                ),
        ),
        Err(_) => IRect::from_corners(IVec2::ZERO, IVec2::ZERO),
    }
}

pub fn position_on_monitor(
    In((surface, monitor, x, y)): In<(Entity, Entity, i32, i32)>,
    mut windows: Query<&mut Window>,
    monitors: Query<(&Monitor, Option<&MonitorWorkarea>)>,
) -> Result<()> {
    let Ok(mut window) = windows.get_mut(surface) else {
        return Ok(());
    };
    let origin = match monitors.get(monitor) {
        Ok((_, Some(workarea))) => workarea.0.min,
        Ok((monitor, None)) => monitor.physical_position,
        Err(_) => return Ok(()),
    };
    window.position = WindowPosition::At(origin + IVec2::new(x, y));
    Ok(())
}

pub fn center_on_monitor(
    In((surface, monitor)): In<(Entity, Entity)>,
    mut windows: Query<&mut Window>,
    monitors: Query<(&Monitor, Option<&MonitorWorkarea>)>,
) -> Result<()> {
    let Ok(mut window) = windows.get_mut(surface) else {
        return Ok(());
    };
    let area = match monitors.get(monitor) {
        Ok((_, Some(workarea))) => workarea.0,
        Ok((monitor, None)) => IRect::from_corners(
            monitor.physical_position,
            monitor.physical_position
                + IVec2::new(
                    monitor.physical_width as i32,
                    monitor.physical_height as i32,
                ),
        ),
        Err(_) => return Ok(()),
    };
    let size = IVec2::new(
        window.resolution.physical_width() as i32,
        window.resolution.physical_height() as i32,
    );
    window.position = WindowPosition::At(area.min + (area.size() - size) / 2);
    Ok(())
}
