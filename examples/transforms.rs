mod glfw;

use std::f32::consts::PI;

use glfw::GlfwContext;
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    sketch().unwrap();
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(400, 400)?;
    init()?;

    let surface = glfw_ctx.create_surface(400, 400, 1.0)?;
    let graphics = graphics_create(surface, 400, 400)?;

    let mut t: f32 = 0.0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.1, 0.1, 0.1)),
        )?;

        for i in 0..4 {
            for j in 0..4 {
                graphics_record_command(graphics, DrawCommand::PushMatrix)?;

                graphics_record_command(
                    graphics,
                    DrawCommand::Translate {
                        x: 50.0 + j as f32 * 100.0,
                        y: 50.0 + i as f32 * 100.0,
                    },
                )?;

                let angle = t + (i + j) as f32 * PI / 8.0;
                graphics_record_command(graphics, DrawCommand::Rotate { angle })?;

                let s = 0.8 + (t * 2.0 + (i * j) as f32).sin() * 0.2;
                graphics_record_command(graphics, DrawCommand::Scale { x: s, y: s })?;

                let r = j as f32 / 3.0;
                let g = i as f32 / 3.0;
                graphics_record_command(
                    graphics,
                    DrawCommand::Fill(bevy::color::Color::srgb(r, g, 0.8)),
                )?;

                graphics_record_command(
                    graphics,
                    DrawCommand::Rect {
                        x: -20.0,
                        y: -20.0,
                        w: 40.0,
                        h: 40.0,
                        radii: [0.0; 4],
                    },
                )?;

                graphics_record_command(graphics, DrawCommand::PopMatrix)?;
            }
        }

        graphics_end_draw(graphics)?;
        t += 0.02;
    }

    Ok(())
}
