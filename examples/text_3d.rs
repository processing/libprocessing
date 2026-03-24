use processing_glfw::GlfwContext;

use bevy::math::{Vec2, Vec3};
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    sketch().unwrap();
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let width = 1200;
    let height = 700;
    let mut glfw_ctx = GlfwContext::new(width, height)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(width, height)?;
    let graphics = graphics_create(surface, width, height, TextureFormat::Rgba16Float)?;

    // 3D camera
    graphics_mode_3d(graphics)?;
    transform_set_position(graphics, Vec3::new(0.0, 0.0, 800.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    // Directional light to reveal 3D shape
    let dir_light =
        light_create_directional(graphics, bevy::color::Color::srgb(0.3, 0.3, 0.4), 2000.0)?;
    transform_set_position(dir_light, Vec3::new(200.0, 300.0, 500.0))?;
    transform_look_at(dir_light, Vec3::new(0.0, 0.0, 0.0))?;

    // Emissive glowing material
    let glow = material_create_pbr()?;
    material_set(glow, "roughness", shader_value::ShaderValue::Float(0.3))?;
    material_set(glow, "metallic", shader_value::ShaderValue::Float(0.5))?;
    // HDR emissive — values > 1.0 produce bloom/glow
    material_set(glow, "emissive", shader_value::ShaderValue::Float4([2.0, 0.5, 3.0, 1.0]))?;

    // Generate 3D text geometry — measure width to center the mesh
    graphics_record_command(graphics, DrawCommand::TextSize(120.0))?;
    graphics_record_command(graphics, DrawCommand::TextStyle(TextStyle::Bold))?;

    let w = graphics_text_width(graphics, "Processing")?;
    let mesh = graphics_text_to_model(graphics, "Processing", -w / 2.0, 0.0, 40.0)?;
    let geom = geometry_create_from_mesh(mesh)?;

    let mut t: f32 = 0.0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.02, 0.02, 0.03)),
        )?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgb(0.8, 0.3, 1.0)),
        )?;
        graphics_record_command(graphics, DrawCommand::Material(glow))?;

        graphics_record_command(graphics, DrawCommand::PushMatrix)?;
        graphics_record_command(graphics, DrawCommand::Scale(Vec2::new(15.0, 15.0)))?;
        graphics_record_command(graphics, DrawCommand::Rotate { angle: t * 0.3 })?;
        graphics_record_command(graphics, DrawCommand::Geometry(geom))?;
        graphics_record_command(graphics, DrawCommand::PopMatrix)?;

        graphics_end_draw(graphics)?;
        t += 0.016;
    }

    material_destroy(glow)?;

    Ok(())
}
