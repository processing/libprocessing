use processing_glfw::GlfwContext;
use std::f32::consts::PI;

use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    sketch().unwrap();
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(600, 600)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(600, 600)?;
    let graphics = graphics_create(surface, 600, 600, TextureFormat::Rgba16Float)?;

    let mut t: f32 = 0.0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.95, 0.93, 0.9)),
        )?;

        let cx = 300.0;
        let cy = 300.0;

        graphics_record_command(graphics, DrawCommand::NoFill)?;
        graphics_record_command(graphics, DrawCommand::StrokeWeight(2.0))?;

        for i in 0..12 {
            let r = 40.0 + i as f32 * 18.0;
            let offset = t + i as f32 * 0.15;
            let v = 0.2 + (i as f32 / 12.0) * 0.4;
            graphics_record_command(
                graphics,
                DrawCommand::StrokeColor(bevy::color::Color::srgb(v, v * 0.8, v * 0.6)),
            )?;
            graphics_record_command(
                graphics,
                DrawCommand::Arc {
                    cx,
                    cy,
                    w: r * 2.0,
                    h: r * 2.0,
                    start: offset,
                    stop: offset + PI * 1.2,
                    mode: ArcMode::Open,
                },
            )?;
        }

        graphics_record_command(graphics, DrawCommand::NoStroke)?;
        for i in 0..12 {
            let r = 40.0 + i as f32 * 18.0;
            let angle = t + i as f32 * 0.15;
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            graphics_record_command(
                graphics,
                DrawCommand::Fill(bevy::color::Color::srgb(0.85, 0.3 + i as f32 / 24.0, 0.2)),
            )?;
            graphics_record_command(
                graphics,
                DrawCommand::Ellipse {
                    cx: x,
                    cy: y,
                    w: 6.0,
                    h: 6.0,
                },
            )?;
        }

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgba(0.1, 0.1, 0.1, 0.15)),
        )?;
        for i in 0..6 {
            let angle = t * 0.5 + i as f32 * PI / 3.0;
            let d = 250.0;
            let px = cx + d * angle.cos();
            let py = cy + d * angle.sin();
            let s = 20.0;
            graphics_record_command(
                graphics,
                DrawCommand::Triangle {
                    x1: px,
                    y1: py - s,
                    x2: px - s * 0.866,
                    y2: py + s * 0.5,
                    x3: px + s * 0.866,
                    y3: py + s * 0.5,
                },
            )?;
        }

        let d = 15.0 + 5.0 * (t * 3.0).sin();
        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgba(0.9, 0.4, 0.1, 0.4)),
        )?;
        graphics_record_command(
            graphics,
            DrawCommand::Quad {
                x1: cx,
                y1: cy - d,
                x2: cx + d,
                y2: cy,
                x3: cx,
                y3: cy + d,
                x4: cx - d,
                y4: cy,
            },
        )?;

        graphics_end_draw(graphics)?;
        t += 0.01;
    }

    Ok(())
}
