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
#[cfg(any(target_os = "linux", target_arch = "wasm32"))]
use std::ffi::c_void;
use std::ptr::NonNull;

use bevy::{
    app::{App, Plugin},
    asset::Assets,
    ecs::query::QueryEntityError,
    prelude::{Commands, Component, Entity, In, Query, ResMut, Window, With, default},
    render::render_resource::{Extent3d, TextureFormat},
    window::{RawHandleWrapper, WindowResolution, WindowWrapper},
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};

use crate::{
    error,
    error::{ProcessingError, Result},
    image::{Image, ImageTextures},
};

#[derive(Component, Debug, Clone)]
pub struct Surface;

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

    Ok(commands
        .spawn((
            Window {
                resolution: WindowResolution::new(width, height)
                    .with_scale_factor_override(scale_factor),
                ..default()
            },
            handle_wrapper,
            Surface,
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
    let window_handle_ptr = NonNull::new(window_handle as *mut c_void).unwrap();
    let window = WaylandWindowHandle::new(window_handle_ptr);

    if display_handle == 0 {
        return Err(error::ProcessingError::HandleError(
            HandleError::Unavailable,
        ));
    }
    let display_handle_ptr = NonNull::new(display_handle as *mut c_void).unwrap();
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
    mut p_image_textures: ResMut<ImageTextures>,
) -> Result<()> {
    match p_images.get(surface_entity) {
        Ok(p_image) => {
            images.remove(&p_image.handle);
            p_image_textures.remove(&surface_entity);
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

/// Update window size when resized.
pub fn resize(
    In((window_entity, width, height)): In<(Entity, u32, u32)>,
    mut windows: Query<&mut Window>,
) -> Result<()> {
    if let Ok(mut window) = windows.get_mut(window_entity) {
        window.resolution.set_physical_resolution(width, height);

        Ok(())
    } else {
        Err(error::ProcessingError::SurfaceNotFound)
    }
}
