//! Stress test: a "silly" number of PBR-lit cubes slowly rotating. Mostly here
//! to feel out the practical upper bound — change `GRID` to push it harder.

use processing_glfw::GlfwContext;

use bevy::math::Vec3;
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

const GRID: u32 = 100; // GRID^3 = 1,000,000 particles
const SPACING: f32 = 1.0;

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
    if i >= count { return; }
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

fn hash_u32(mut x: u32) -> u32 {
    x = (x ^ 61).wrapping_add(x >> 16);
    x = x.wrapping_add(x << 3);
    x ^= x >> 4;
    x = x.wrapping_mul(0x27d4eb2d);
    x ^= x >> 15;
    x
}

fn hash_unit(seed: u32) -> f32 {
    (hash_u32(seed) as f32) / (u32::MAX as f32)
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(900, 700)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(900, 700)?;
    let graphics = graphics_create(surface, 900, 700, TextureFormat::Rgba16Float)?;

    graphics_mode_3d(graphics)?;
    let extent = (GRID as f32) * SPACING * 0.5;
    transform_set_position(graphics, Vec3::new(0.0, extent * 0.6, extent * 2.5))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;
    graphics_orbit_camera(graphics)?;

    // Three directional R/G/B lights from cardinal axes — each cube face picks
    // up the closest light's color so the lighting variation is obvious.
    let red = light_create_directional(graphics, bevy::color::Color::srgb(1.0, 0.0, 0.0), 1000.0)?;
    transform_set_position(red, Vec3::new(1.0, 0.0, 0.0))?;
    transform_look_at(red, Vec3::ZERO)?;

    let green =
        light_create_directional(graphics, bevy::color::Color::srgb(0.0, 1.0, 0.0), 1000.0)?;
    transform_set_position(green, Vec3::new(0.0, 1.0, 0.0))?;
    transform_look_at(green, Vec3::ZERO)?;

    let blue = light_create_directional(graphics, bevy::color::Color::srgb(0.0, 0.0, 1.0), 1000.0)?;
    transform_set_position(blue, Vec3::new(0.0, 0.0, 1.0))?;
    transform_look_at(blue, Vec3::ZERO)?;

    let cube = geometry_box(0.35, 0.35, 0.35)?;

    let capacity = GRID * GRID * GRID;
    let position_attr = geometry_attribute_position();
    let color_attr = geometry_attribute_color();
    let field = field_create(capacity, vec![position_attr, color_attr])?;

    let mut positions: Vec<f32> = Vec::with_capacity(capacity as usize * 3);
    let mut colors: Vec<f32> = Vec::with_capacity(capacity as usize * 4);
    let extent_half = (GRID as f32) * SPACING * 0.5;
    for i in 0..capacity {
        // Three independent hash streams give us pseudo-random uniform values.
        let rx = hash_unit(i.wrapping_mul(2654435761).wrapping_add(0x9E37));
        let ry = hash_unit(i.wrapping_mul(40503).wrapping_add(0x68E1));
        let rz = hash_unit(i.wrapping_mul(2246822519).wrapping_add(0xC2B2));
        positions.push((rx * 2.0 - 1.0) * extent_half);
        positions.push((ry * 2.0 - 1.0) * extent_half);
        positions.push((rz * 2.0 - 1.0) * extent_half);
        // Color from the same random samples — stable per particle.
        colors.push(rx);
        colors.push(ry);
        colors.push(rz);
        colors.push(1.0);
    }
    let position_buf = field_buffer(field, position_attr)?
        .ok_or(error::ProcessingError::FieldNotFound)?;
    let color_buf = field_buffer(field, color_attr)?
        .ok_or(error::ProcessingError::FieldNotFound)?;
    buffer_write(
        position_buf,
        positions.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;
    buffer_write(
        color_buf,
        colors.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;

    let mat = material_create_field_pbr(color_buf)?;
    let spin_shader = shader_create(SPIN_SHADER)?;
    let spin = compute_create(spin_shader)?;

    eprintln!("field_stress: {capacity} particles");

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.04, 0.04, 0.07)),
        )?;
        graphics_record_command(graphics, DrawCommand::Material(mat))?;
        graphics_record_command(
            graphics,
            DrawCommand::Field {
                field,
                geometry: cube,
            },
        )?;
        graphics_end_draw(graphics)?;

        compute_set(spin, "dt", shader_value::ShaderValue::Float(0.003))?;
        field_apply(field, spin)?;
    }

    Ok(())
}
