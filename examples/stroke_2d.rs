mod glfw;

use glfw::GlfwContext;
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    sketch().unwrap();
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(600, 300)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(600, 300)?;
    let graphics = graphics_create(surface, 600, 300, TextureFormat::Rgba16Float)?;

    let joins = [
        StrokeJoinMode::Round,
        StrokeJoinMode::Miter,
        StrokeJoinMode::Bevel,
    ];

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.15, 0.15, 0.2)),
        )?;

        graphics_record_command(graphics, DrawCommand::StrokeWeight(12.0))?;

        for (i, &join) in joins.iter().enumerate() {
            let x = 30.0 + i as f32 * 190.0;
            let y = 50.0;

            graphics_record_command(graphics, DrawCommand::StrokeJoin(join))?;

            let hue = i as f32 * 0.3;
            graphics_record_command(
                graphics,
                DrawCommand::Fill(bevy::color::Color::srgb(0.3 + hue, 0.5, 0.8 - hue)),
            )?;
            graphics_record_command(
                graphics,
                DrawCommand::StrokeColor(bevy::color::Color::srgb(1.0, 0.9, 0.3)),
            )?;

            graphics_record_command(
                graphics,
                DrawCommand::Rect {
                    x,
                    y,
                    w: 150.0,
                    h: 180.0,
                    radii: [0.0; 4],
                },
            )?;
        }

        graphics_end_draw(graphics)?;
    }

    Ok(())
}
