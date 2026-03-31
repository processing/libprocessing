use processing_glfw::GlfwContext;

use bevy::math::{Vec2, Vec3};
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    sketch().unwrap();
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(900, 400)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(900, 400)?;
    let graphics = graphics_create(surface, 900, 400, TextureFormat::Rgba16Float)?;

    graphics_mode_3d(graphics)?;
    transform_set_position(graphics, Vec3::new(0.0, 80.0, 800.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    let _light =
        light_create_directional(graphics, bevy::color::Color::srgb(0.9, 0.85, 0.8), 300.0)?;

    let pbr = material_create_pbr()?;
    material_set(pbr, "roughness", material::MaterialValue::Float(0.35))?;

    let mut t: f32 = 0.0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.06, 0.06, 0.08)),
        )?;

        graphics_record_command(graphics, DrawCommand::Fill(bevy::color::Color::WHITE))?;
        graphics_record_command(graphics, DrawCommand::Material(pbr))?;

        let shapes: Vec<(f32, Box<dyn Fn(f32) -> DrawCommand>)> = vec![
            (
                -315.0,
                Box::new(|_| DrawCommand::Box {
                    width: 50.0,
                    height: 50.0,
                    depth: 50.0,
                }),
            ),
            (
                -225.0,
                Box::new(|_| DrawCommand::Sphere {
                    radius: 30.0,
                    sectors: 24,
                    stacks: 16,
                }),
            ),
            (
                -135.0,
                Box::new(|_| DrawCommand::Cylinder {
                    radius: 25.0,
                    height: 60.0,
                    detail: 24,
                }),
            ),
            (
                -45.0,
                Box::new(|_| DrawCommand::Cone {
                    radius: 25.0,
                    height: 60.0,
                    detail: 24,
                }),
            ),
            (
                45.0,
                Box::new(|_| DrawCommand::Torus {
                    radius: 30.0,
                    tube_radius: 10.0,
                    major_segments: 24,
                    minor_segments: 16,
                }),
            ),
            (
                135.0,
                Box::new(|_| DrawCommand::Capsule {
                    radius: 15.0,
                    length: 40.0,
                    detail: 24,
                }),
            ),
            (
                225.0,
                Box::new(|_| DrawCommand::ConicalFrustum {
                    radius_top: 15.0,
                    radius_bottom: 25.0,
                    height: 60.0,
                    detail: 24,
                }),
            ),
            (
                315.0,
                Box::new(|_| DrawCommand::Tetrahedron { radius: 30.0 }),
            ),
        ];

        for (x_offset, make_cmd) in &shapes {
            graphics_record_command(graphics, DrawCommand::PushMatrix)?;
            graphics_record_command(graphics, DrawCommand::Translate(Vec2::new(*x_offset, 0.0)))?;
            graphics_record_command(graphics, DrawCommand::Rotate { angle: t })?;
            graphics_record_command(graphics, make_cmd(t))?;
            graphics_record_command(graphics, DrawCommand::PopMatrix)?;
        }

        graphics_end_draw(graphics)?;
        t += 0.015;
    }

    Ok(())
}
