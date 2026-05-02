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
    transform_set_position(graphics, Vec3::new(0.0, 4.0, 14.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    let sphere = geometry_sphere(0.08, 8, 6)?;

    let capacity: u32 = 2000;
    let position_attr = geometry_attribute_position();
    let color_attr = geometry_attribute_color();
    let field = field_create(capacity, vec![position_attr, color_attr])?;
    let position_buf = field_buffer(field, position_attr)?
        .ok_or(error::ProcessingError::FieldNotFound)?;
    let color_buf = field_buffer(field, color_attr)?
        .ok_or(error::ProcessingError::FieldNotFound)?;

    // Push unemitted slots far off-screen so they don't all render at the
    // origin while the ring buffer is still filling.
    let init_positions: Vec<f32> = (0..capacity * 3).map(|_| 1.0e6).collect();
    buffer_write(
        position_buf,
        init_positions.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;

    let mat = material_create_field_color(color_buf)?;

    let mut frame: u32 = 0;
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
                geometry: sphere,
            },
        )?;
        graphics_end_draw(graphics)?;

        // Emit 4 particles per frame in an outward-spiraling ring; once the ring
        // buffer fills (~500 frames at 4/frame for capacity 2000), oldest get
        // overwritten and the swirl continues without bound.
        let burst = 4u32;
        let mut positions: Vec<f32> = Vec::with_capacity(burst as usize * 3);
        let mut colors: Vec<f32> = Vec::with_capacity(burst as usize * 4);
        for k in 0..burst {
            let i = frame * burst + k;
            let t = i as f32 * 0.05;
            let radius = 1.5 + (t * 0.02).min(3.0);
            let height = ((t * 0.1).sin()) * 2.0;
            positions.push(t.cos() * radius);
            positions.push(height);
            positions.push(t.sin() * radius);
            // Hue sweep based on emission index.
            let h = (i as f32 * 0.012) % 1.0;
            let (r, g, b) = hsv_to_rgb(h, 0.85, 1.0);
            colors.push(r);
            colors.push(g);
            colors.push(b);
            colors.push(1.0);
        }

        let position_bytes: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        let color_bytes: Vec<u8> = colors.iter().flat_map(|f| f.to_le_bytes()).collect();
        field_emit(
            field,
            burst,
            vec![(position_attr, position_bytes), (color_attr, color_bytes)],
        )?;
        frame += 1;
    }

    Ok(())
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match (i as i32) % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
