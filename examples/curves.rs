use processing_glfw::GlfwContext;

use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    sketch().unwrap();
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(800, 400)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(800, 400)?;
    let graphics = graphics_create(surface, 800, 400, TextureFormat::Rgba16Float)?;

    let mut t: f32 = 0.0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.04, 0.04, 0.06)),
        )?;

        graphics_record_command(graphics, DrawCommand::NoFill)?;

        graphics_record_command(graphics, DrawCommand::StrokeWeight(1.5))?;
        for i in 0..20 {
            let y_base = 20.0 + i as f32 * 15.0;
            let phase = t + i as f32 * 0.3;
            let amp = 30.0 + (i as f32 * 0.5).sin() * 20.0;

            let v = 0.3 + (i as f32 / 20.0) * 0.5;
            graphics_record_command(
                graphics,
                DrawCommand::StrokeColor(bevy::color::Color::srgb(v * 0.6, v, (v * 1.2).min(1.0))),
            )?;

            graphics_record_command(
                graphics,
                DrawCommand::Bezier {
                    x1: 0.0,
                    y1: y_base,
                    x2: 250.0,
                    y2: y_base + amp * phase.sin(),
                    x3: 550.0,
                    y3: y_base - amp * (phase * 1.3).cos(),
                    x4: 800.0,
                    y4: y_base,
                },
            )?;
        }

        graphics_record_command(graphics, DrawCommand::StrokeWeight(0.8))?;
        for i in 0..8 {
            let y_base = 50.0 + i as f32 * 40.0;
            let phase = t * 0.7 + i as f32 * 0.5;

            graphics_record_command(
                graphics,
                DrawCommand::StrokeColor(bevy::color::Color::srgba(1.0, 0.6, 0.2, 0.4)),
            )?;

            graphics_record_command(
                graphics,
                DrawCommand::Curve {
                    x1: -50.0,
                    y1: y_base + 40.0 * phase.sin(),
                    x2: 200.0,
                    y2: y_base + 30.0 * (phase * 1.2).cos(),
                    x3: 600.0,
                    y3: y_base - 30.0 * (phase * 0.8).sin(),
                    x4: 850.0,
                    y4: y_base + 20.0 * phase.cos(),
                },
            )?;
        }

        graphics_record_command(graphics, DrawCommand::StrokeWeight(0.3))?;
        graphics_record_command(
            graphics,
            DrawCommand::StrokeColor(bevy::color::Color::srgba(1.0, 1.0, 1.0, 0.08)),
        )?;
        for i in 0..20 {
            let y = 20.0 * i as f32;
            graphics_record_command(
                graphics,
                DrawCommand::Line {
                    x1: 0.0,
                    y1: y,
                    x2: 800.0,
                    y2: y,
                },
            )?;
        }

        graphics_end_draw(graphics)?;
        t += 0.012;
    }

    Ok(())
}
