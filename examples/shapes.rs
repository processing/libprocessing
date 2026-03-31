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
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.92, 0.90, 0.87)),
        )?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgba(0.2, 0.3, 0.5, 0.25)),
        )?;
        graphics_record_command(graphics, DrawCommand::StrokeWeight(1.5))?;
        graphics_record_command(
            graphics,
            DrawCommand::StrokeColor(bevy::color::Color::srgb(0.15, 0.2, 0.35)),
        )?;

        let n = 8;
        graphics_record_command(
            graphics,
            DrawCommand::BeginShape {
                kind: ShapeKind::Polygon,
            },
        )?;

        for i in 0..n + 3 {
            let idx = (i + n - 1) % n;
            let angle = idx as f32 * 2.0 * PI / n as f32;
            let r = 100.0 + 30.0 * (t * 2.0 + idx as f32 * 0.8).sin();
            graphics_record_command(
                graphics,
                DrawCommand::ShapeCurveVertex {
                    x: 170.0 + r * angle.cos(),
                    y: 200.0 + r * angle.sin(),
                },
            )?;
        }

        graphics_record_command(graphics, DrawCommand::EndShape { close: true })?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgba(0.15, 0.5, 0.3, 0.2)),
        )?;
        graphics_record_command(graphics, DrawCommand::StrokeWeight(1.0))?;
        graphics_record_command(
            graphics,
            DrawCommand::StrokeColor(bevy::color::Color::srgb(0.1, 0.35, 0.2)),
        )?;

        let pcx = 430.0;
        let pcy = 200.0;
        let s = 70.0 + 10.0 * (t * 1.5).sin();

        graphics_record_command(
            graphics,
            DrawCommand::BeginShape {
                kind: ShapeKind::Polygon,
            },
        )?;
        graphics_record_command(graphics, DrawCommand::ShapeVertex { x: pcx, y: pcy - s })?;
        graphics_record_command(
            graphics,
            DrawCommand::ShapeBezierVertex {
                cx1: pcx + s,
                cy1: pcy - s,
                cx2: pcx + s,
                cy2: pcy + s,
                x: pcx,
                y: pcy + s,
            },
        )?;
        graphics_record_command(
            graphics,
            DrawCommand::ShapeBezierVertex {
                cx1: pcx - s,
                cy1: pcy + s,
                cx2: pcx - s,
                cy2: pcy - s,
                x: pcx,
                y: pcy - s,
            },
        )?;
        graphics_record_command(graphics, DrawCommand::EndShape { close: true })?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgba(0.8, 0.35, 0.1, 0.15)),
        )?;
        graphics_record_command(graphics, DrawCommand::NoStroke)?;

        graphics_record_command(
            graphics,
            DrawCommand::BeginShape {
                kind: ShapeKind::TriangleFan,
            },
        )?;
        graphics_record_command(graphics, DrawCommand::ShapeVertex { x: 170.0, y: 450.0 })?;
        for i in 0..=16 {
            let angle = i as f32 * 2.0 * PI / 16.0;
            let r = 60.0 + 20.0 * (t * 3.0 + angle * 2.0).sin();
            graphics_record_command(
                graphics,
                DrawCommand::ShapeVertex {
                    x: 170.0 + r * angle.cos(),
                    y: 450.0 + r * angle.sin(),
                },
            )?;
        }
        graphics_record_command(graphics, DrawCommand::EndShape { close: true })?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgba(0.5, 0.2, 0.6, 0.2)),
        )?;
        graphics_record_command(graphics, DrawCommand::StrokeWeight(0.5))?;
        graphics_record_command(
            graphics,
            DrawCommand::StrokeColor(bevy::color::Color::srgba(0.3, 0.1, 0.4, 0.4)),
        )?;

        graphics_record_command(
            graphics,
            DrawCommand::BeginShape {
                kind: ShapeKind::TriangleStrip,
            },
        )?;
        for i in 0..16 {
            let x = 320.0 + i as f32 * 17.0;
            let wave = 30.0 * (t * 2.0 + i as f32 * 0.4).sin();
            graphics_record_command(graphics, DrawCommand::ShapeVertex { x, y: 420.0 + wave })?;
            graphics_record_command(graphics, DrawCommand::ShapeVertex { x, y: 480.0 + wave })?;
        }
        graphics_record_command(graphics, DrawCommand::EndShape { close: false })?;

        graphics_end_draw(graphics)?;
        t += 0.02;
    }

    Ok(())
}
