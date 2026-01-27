mod glfw;

use glfw::GlfwContext;
use processing::prelude::*;
use processing_render::{render::command::DrawCommand};

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
    init(Config::default())?;

    let width = 400;
    let height = 400;
    let scale_factor = 1.0;

    let surface = glfw_ctx.create_surface(width, height, scale_factor)?;
    let graphics = graphics_create(surface, width, height)?;

    graphics_mode_3d(graphics)?;
    graphics_camera_position(graphics, 100.0, 100.0, 300.0)?;
    graphics_camera_look_at(graphics, 0.0, 0.0, 0.0)?;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.1, 0.1, 0.15)),
        )?;
        
        geometry_begin(graphics)?;
        
        graphics_record_command(graphics, DrawCommand::PushMatrix)?;
        geometry_sphere(graphics, 10.)?;
        graphics_record_command(graphics, DrawCommand::PopMatrix)?;
        
        let geometry = geometry_end(graphics)?;
        graphics_record_command(graphics, DrawCommand::Geometry(geometry))?;
        graphics_end_draw(graphics)?;
    }
    Ok(())
}
