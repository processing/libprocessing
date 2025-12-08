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
    pub fn get_window(&self) -> u64 {
        self.window.get_cocoa_window() as u64
    }

    #[cfg(target_os = "windows")]
    pub fn get_window(&self) -> u64 {
        self.window.get_win32_window() as u64
    }

    #[cfg(target_os = "linux")]
    pub fn get_window(&self) -> u64 {
        self.window.get_wayland_window() as u64
    }

    #[cfg(not(target_os = "linux"))]
    pub fn get_display(&self) -> u64 {
        0
    }

    #[cfg(target_os = "linux")]
    pub fn get_display(&self) -> u64 {
        self.glfw.get_wayland_display() as u64
    }

    pub fn poll_events(&mut self) -> bool {
        self.glfw.poll_events();

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Close => return false,
                _ => {}
            }
        }

        !self.window.should_close()
    }
}
