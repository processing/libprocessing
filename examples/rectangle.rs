mod glfw;

use glfw::GlfwContext;
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    match sketch() {
        Ok(_) => {
            eprintln!("Sketch completed successfully");
            exit(0).unwrap();
        }
        Err(e) => {
            eprintln!("Sketch error: {:?}", e);
            exit(1).unwrap();
        }
    };
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(400, 400)?;
    init()?;

    let width = 400;
    let height = 400;
    let scale_factor = 1.0;

    let window_handle = glfw_ctx.get_window();
    let display_handle = glfw_ctx.get_display();
    let surface = surface_create(window_handle, display_handle, width, height, scale_factor)?;
    let graphics = graphics_create(surface, width, height)?;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::Rect {
                x: 10.0,
                y: 10.0,
                w: 100.0,
                h: 100.0,
                radii: [0.0, 0.0, 0.0, 0.0],
            },
        )?;

        graphics_end_draw(graphics)?;
    }
    Ok(())
}
