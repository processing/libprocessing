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
    transform_set_position(graphics, Vec3::new(0.0, 4.0, 18.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    let _light =
        light_create_directional(graphics, bevy::color::Color::srgb(0.95, 0.9, 0.85), 200.0)?;

    // Source mesh whose vertices become the particle positions. UVs come along
    // for free and we'll use them to paint each particle a unique color.
    let source = geometry_sphere(5.0, 32, 24)?;

    let position_attr = geometry_attribute_position();
    let uv_attr = geometry_attribute_uv();
    let color_attr = geometry_attribute_color();

    // Position + uv come straight from the source sphere; color is allocated
    // empty and we fill it from uv values.
    let field = field_create_from_geometry(source, vec![position_attr, uv_attr, color_attr])?;
    let uv_buf =
        field_buffer(field, uv_attr)?.ok_or(error::ProcessingError::FieldNotFound)?;
    let color_buf =
        field_buffer(field, color_attr)?.ok_or(error::ProcessingError::FieldNotFound)?;

    // Read uvs back, build per-particle colors from them, write to color buffer.
    let uv_bytes = buffer_read(uv_buf)?;
    let mut colors: Vec<u8> = Vec::with_capacity(uv_bytes.len() * 2);
    for chunk in uv_bytes.chunks_exact(8) {
        let u = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let v = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        let (r, g, b) = hsv_to_rgb(u, 0.85, 1.0);
        for f in [r, g, b, 1.0] {
            colors.extend_from_slice(&f.to_le_bytes());
        }
        let _ = v;
    }
    buffer_write(color_buf, colors)?;

    let particle = geometry_sphere(0.18, 10, 8)?;
    let mat = material_create_field_pbr(color_buf)?;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.06, 0.06, 0.08)),
        )?;
        graphics_record_command(graphics, DrawCommand::Material(mat))?;
        graphics_record_command(
            graphics,
            DrawCommand::Field {
                field,
                geometry: particle,
            },
        )?;
        graphics_end_draw(graphics)?;
    }

    Ok(())
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match (i as i32).rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
