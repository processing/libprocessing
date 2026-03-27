use processing_glfw::GlfwContext;

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
    let width = 400;
    let height = 400;
    let mut glfw_ctx = GlfwContext::new(width, height)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(width, height)?;
    let graphics = graphics_create(surface, width, height, TextureFormat::Rgba16Float)?;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        let mx = input_mouse_x(surface)?;
        let my = input_mouse_y(surface)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.15, 0.15, 0.2)),
        )?;

        if input_mouse_is_pressed()? {
            graphics_record_command(
                graphics,
                DrawCommand::Fill(bevy::color::Color::srgb(0.25, 0.15, 0.2)),
            )?;
        } else {
        }

        if input_mouse_is_pressed()? {
            graphics_record_command(
                graphics,
                DrawCommand::Fill(bevy::color::Color::srgb(0.55, 0.25, 0.2)),
            )?;
        } else {
        }

        graphics_record_command(
            graphics,
            DrawCommand::Rect {
                x: mx - 25.0,
                y: my - 25.0,
                w: 50.0,
                h: 50.0,
                radii: [0.0; 4],
            },
        )?;

        if input_key_is_pressed()? && input_key_is_down(KeyCode::Escape)? {
            break;
        }

        graphics_end_draw(graphics)?;
    }
    Ok(())
}
