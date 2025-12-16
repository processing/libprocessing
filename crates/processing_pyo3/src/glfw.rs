use bevy::prelude::Entity;
/// Minimal GLFW helper for Processing examples
use glfw::{Glfw, GlfwReceiver, PWindow, WindowEvent, WindowMode};
use processing::prelude::error::Result;

pub struct GlfwContext {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
}

impl GlfwContext {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfw.window_hint(glfw::WindowHint::Visible(false));

        let (mut window, events) = glfw
            .create_window(width, height, "Processing", WindowMode::Windowed)
            .unwrap();

        window.set_all_polling(true);
        window.show();

        Ok(Self {
            glfw,
            window,
            events,
        })
    }

    #[cfg(target_os = "macos")]
    pub fn create_surface(&self, width: u32, height: u32, scale_factor: f32) -> Result<Entity> {
        use processing::prelude::surface_create_macos;
        surface_create_macos(
            self.window.get_cocoa_window() as u64,
            width,
            height,
            scale_factor,
        )
    }

    #[cfg(target_os = "windows")]
    pub fn create_surface(&self, width: u32, height: u32, scale_factor: f32) -> Result<Entity> {
        use processing::prelude::surface_create_windows;
        surface_create_windows(
            self.window.get_win32_window() as u64,
            width,
            height,
            scale_factor,
        )
    }

    #[cfg(all(target_os = "linux", feature = "wayland"))]
    pub fn create_surface(&self, width: u32, height: u32, scale_factor: f32) -> Result<Entity> {
        use processing::prelude::surface_create_wayland;
        surface_create_wayland(
            self.window.get_wayland_window() as u64,
            self.glfw.get_wayland_display() as u64,
            width,
            height,
            scale_factor,
        )
    }

    #[cfg(all(target_os = "linux", feature = "x11"))]
    pub fn create_surface(&self, width: u32, height: u32, scale_factor: f32) -> Result<Entity> {
        use processing::prelude::surface_create_x11;
        surface_create_x11(
            self.window.get_x11_window() as u64,
            self.glfw.get_x11_display() as u64,
            width,
            height,
            scale_factor,
        )
    }

    pub fn poll_events(&mut self) -> bool {
        self.glfw.poll_events();

        for (_, event) in glfw::flush_messages(&self.events) {
            if event == WindowEvent::Close {
                return false;
            }
        }

        !self.window.should_close()
    }
}
