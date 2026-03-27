use processing_glfw::GlfwContext;

use bevy::math::{Vec2, Vec3};
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    sketch().unwrap();
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(600, 400)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(600, 400)?;
    let graphics = graphics_create(surface, 600, 400, TextureFormat::Rgba16Float)?;

    graphics_mode_3d(graphics)?;
    transform_set_position(graphics, Vec3::new(200.0, 150.0, 350.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    let _light =
        light_create_directional(graphics, bevy::color::Color::srgb(0.9, 0.85, 0.8), 800.0)?;

    let mut angle: f32 = 0.0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.08, 0.08, 0.12)),
        )?;

        // thin wireframe box
        graphics_record_command(graphics, DrawCommand::PushMatrix)?;
        graphics_record_command(graphics, DrawCommand::Translate(Vec2::new(-80.0, 0.0)))?;
        graphics_record_command(graphics, DrawCommand::Rotate { angle })?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgb(0.3, 0.4, 0.7)),
        )?;
        graphics_record_command(
            graphics,
            DrawCommand::StrokeColor(bevy::color::Color::srgb(1.0, 1.0, 1.0)),
        )?;
        graphics_record_command(graphics, DrawCommand::StrokeWeight(1.0))?;

        graphics_record_command(
            graphics,
            DrawCommand::Box {
                width: 60.0,
                height: 60.0,
                depth: 60.0,
            },
        )?;
        graphics_record_command(graphics, DrawCommand::PopMatrix)?;

        // thick wireframe box
        graphics_record_command(graphics, DrawCommand::PushMatrix)?;
        graphics_record_command(graphics, DrawCommand::Translate(Vec2::ZERO))?;
        graphics_record_command(graphics, DrawCommand::Rotate { angle: angle * 0.7 })?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgb(0.7, 0.3, 0.4)),
        )?;
        graphics_record_command(
            graphics,
            DrawCommand::StrokeColor(bevy::color::Color::srgb(1.0, 0.9, 0.2)),
        )?;
        graphics_record_command(graphics, DrawCommand::StrokeWeight(3.0))?;

        graphics_record_command(
            graphics,
            DrawCommand::Box {
                width: 50.0,
                height: 70.0,
                depth: 50.0,
            },
        )?;
        graphics_record_command(graphics, DrawCommand::PopMatrix)?;

        // thick wireframe sphere
        graphics_record_command(graphics, DrawCommand::PushMatrix)?;
        graphics_record_command(graphics, DrawCommand::Translate(Vec2::new(80.0, 0.0)))?;
        graphics_record_command(graphics, DrawCommand::Rotate { angle: angle * 0.5 })?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgb(0.3, 0.7, 0.4)),
        )?;
        graphics_record_command(
            graphics,
            DrawCommand::StrokeColor(bevy::color::Color::srgb(0.9, 0.4, 1.0)),
        )?;
        graphics_record_command(graphics, DrawCommand::StrokeWeight(2.0))?;

        graphics_record_command(
            graphics,
            DrawCommand::Sphere {
                radius: 35.0,
                sectors: 16,
                stacks: 12,
            },
        )?;
        graphics_record_command(graphics, DrawCommand::PopMatrix)?;

        // wireframe-only sphere (no fill)
        graphics_record_command(graphics, DrawCommand::PushMatrix)?;
        graphics_record_command(graphics, DrawCommand::Translate(Vec2::new(160.0, 0.0)))?;
        graphics_record_command(
            graphics,
            DrawCommand::Rotate {
                angle: -angle * 0.3,
            },
        )?;

        graphics_record_command(graphics, DrawCommand::NoFill)?;
        graphics_record_command(
            graphics,
            DrawCommand::StrokeColor(bevy::color::Color::srgb(0.2, 0.8, 1.0)),
        )?;
        graphics_record_command(graphics, DrawCommand::StrokeWeight(1.5))?;

        graphics_record_command(
            graphics,
            DrawCommand::Sphere {
                radius: 30.0,
                sectors: 24,
                stacks: 16,
            },
        )?;
        graphics_record_command(graphics, DrawCommand::PopMatrix)?;

        graphics_end_draw(graphics)?;
        angle += 0.015;
    }

    Ok(())
}
