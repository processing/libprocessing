use processing_glfw::GlfwContext;

use bevy::math::Vec3;
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    sketch().unwrap();
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(900, 700)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(900, 700)?;
    let graphics = graphics_create(surface, 900, 700, TextureFormat::Rgba16Float)?;

    graphics_mode_3d(graphics)?;
    transform_set_position(graphics, Vec3::new(0.0, 0.0, 25.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    let _light =
        light_create_directional(graphics, bevy::color::Color::srgb(0.9, 0.85, 0.8), 300.0)?;

    let sphere = geometry_sphere(0.25, 12, 8)?;

    // 10x10x10 grid of positions in a 9-unit cube centered at the origin.
    let capacity: u32 = 1000;
    let mut floats: Vec<f32> = Vec::with_capacity(capacity as usize * 3);
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                floats.push((x as f32 - 4.5) * 1.0);
                floats.push((y as f32 - 4.5) * 1.0);
                floats.push((z as f32 - 4.5) * 1.0);
            }
        }
    }
    let bytes: Vec<u8> = floats.iter().flat_map(|f| f.to_le_bytes()).collect();

    let position_attr = geometry_attribute_position();
    let p = particles_create(capacity, vec![position_attr])?;
    let position_buf = particles_buffer(p, position_attr)?
        .ok_or(error::ProcessingError::ParticlesNotFound)?;
    buffer_write(position_buf, bytes)?;

    let pbr = material_create_pbr()?;
    material_set(pbr, "roughness", shader_value::ShaderValue::Float(0.4))?;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.06, 0.06, 0.08)),
        )?;
        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgb(0.9, 0.5, 0.3)),
        )?;
        graphics_record_command(graphics, DrawCommand::Material(pbr))?;
        graphics_record_command(
            graphics,
            DrawCommand::Particles { particles: p, geometry: sphere },
        )?;
        graphics_end_draw(graphics)?;
    }

    Ok(())
}
