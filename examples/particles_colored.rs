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
    transform_set_position(graphics, Vec3::new(0.0, 6.0, 18.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    let sphere = geometry_sphere(0.25, 12, 8)?;

    // 10x10x10 grid with per-particle position + color (RGB gradient by index).
    let capacity: u32 = 1000;
    let mut positions: Vec<f32> = Vec::with_capacity(capacity as usize * 3);
    let mut colors: Vec<f32> = Vec::with_capacity(capacity as usize * 4);
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                positions.push((x as f32 - 4.5) * 1.0);
                positions.push((y as f32 - 4.5) * 1.0);
                positions.push((z as f32 - 4.5) * 1.0);
                colors.push(x as f32 / 9.0);
                colors.push(y as f32 / 9.0);
                colors.push(z as f32 / 9.0);
                colors.push(1.0);
            }
        }
    }

    let position_attr = geometry_attribute_position();
    let color_attr = geometry_attribute_color();
    let p = particles_create(capacity, vec![position_attr, color_attr])?;
    let position_buf = particles_buffer(p, position_attr)?
        .ok_or(error::ProcessingError::ParticlesNotFound)?;
    let color_buf = particles_buffer(p, color_attr)?
        .ok_or(error::ProcessingError::ParticlesNotFound)?;
    buffer_write(
        position_buf,
        positions.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;
    buffer_write(
        color_buf,
        colors.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;

    let mat = { let m = material_create_unlit()?; material_set_albedo_buffer(m, color_buf)?; m };

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.06, 0.06, 0.08)),
        )?;
        graphics_record_command(graphics, DrawCommand::Material(mat))?;
        graphics_record_command(
            graphics,
            DrawCommand::Particles { particles: p, geometry: sphere },
        )?;
        graphics_end_draw(graphics)?;
    }

    Ok(())
}
