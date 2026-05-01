use processing_glfw::GlfwContext;

use bevy::math::Vec3;
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

const SPIN_SHADER: &str = r#"
struct Params {
    dt: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count {
        return;
    }
    let cs = cos(params.dt);
    let sn = sin(params.dt);
    let x = position[i * 3u + 0u];
    let z = position[i * 3u + 2u];
    position[i * 3u + 0u] = x * cs - z * sn;
    position[i * 3u + 2u] = x * sn + z * cs;
}
"#;

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
    transform_set_position(graphics, Vec3::new(0.0, 8.0, 25.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    let _light =
        light_create_directional(graphics, bevy::color::Color::srgb(0.9, 0.85, 0.8), 300.0)?;

    let sphere = geometry_sphere(0.25, 12, 8)?;

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
    let field = field_create(capacity, vec![position_attr])?;
    let position_buf = field_pbuffer(field, position_attr)?
        .ok_or(error::ProcessingError::FieldNotFound)?;
    buffer_write(position_buf, bytes)?;

    let pbr = material_create_pbr()?;
    material_set(pbr, "roughness", shader_value::ShaderValue::Float(0.4))?;

    let spin_shader = shader_create(SPIN_SHADER)?;
    let spin = compute_create(spin_shader)?;

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
            DrawCommand::Field {
                field,
                geometry: sphere,
            },
        )?;
        graphics_end_draw(graphics)?;

        compute_set(spin, "dt", shader_value::ShaderValue::Float(0.01))?;
        field_apply(field, spin)?;
    }

    Ok(())
}
