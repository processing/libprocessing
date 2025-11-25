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

    let window_handle = glfw_ctx.get_window();
    let display_handle = glfw_ctx.get_display();
    let surface = create_surface(window_handle, display_handle, 400, 400, 1.0)?;

    while glfw_ctx.poll_events() {
        begin_draw(surface)?;

        record_command(
            surface,
            DrawCommand::Rect {
                x: 10.0,
                y: 10.0,
                w: 100.0,
                h: 100.0,
                radii: [0.0, 0.0, 0.0, 0.0],
            },
        )?;

        end_draw(surface)?;

    }
    Ok(())
}
