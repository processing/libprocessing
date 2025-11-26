mod glfw;

use bevy::prelude::Color;
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
    let surface = surface_create(window_handle, display_handle, 400, 400, 1.0)?;
    let image = image_load("images/logo.png")?;

    while glfw_ctx.poll_events() {
        begin_draw(surface)?;

        record_command(surface, DrawCommand::BackgroundImage(image))?;

        end_draw(surface)?;
    }
    Ok(())
}
