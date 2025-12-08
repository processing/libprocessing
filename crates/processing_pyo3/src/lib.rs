mod glfw;
use pyo3::prelude::*;

#[pymodule]
mod pycessing {
    use crate::glfw::GlfwContext;
    use processing::prelude::*;
    use pyo3::prelude::*;

    /// create surface
    #[pyfunction]
    fn size(width: u32, height: u32) -> PyResult<String> {
        let mut glfw_ctx = GlfwContext::new(400, 400).unwrap();
        init().unwrap();

        let window_handle = glfw_ctx.get_window();
        let display_handle = glfw_ctx.get_display();
        let surface = surface_create(window_handle, display_handle, width, height, 1.0).unwrap();

        while glfw_ctx.poll_events() {
            begin_draw(surface).unwrap();

            record_command(
                surface,
                DrawCommand::Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 100.0,
                    h: 100.0,
                    radii: [0.0, 0.0, 0.0, 0.0],
                },
            )
            .unwrap();

            end_draw(surface).unwrap();
        }

        Ok("OK".to_string())
    }
}
