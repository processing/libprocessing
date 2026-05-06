use processing_glfw::GlfwContext;

use bevy::math::Vec3;
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

const SPIN_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read_write> rotation: array<f32>;
@group(0) @binding(1) var<uniform> params: vec4<f32>;  // x = dt

fn quat_mul(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        a.w * b.xyz + b.w * a.xyz + cross(a.xyz, b.xyz),
        a.w * b.w - dot(a.xyz, b.xyz),
    );
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&rotation) / 4u;
    if i >= count {
        return;
    }
    let dt = params.x;
    let q = vec4<f32>(
        rotation[i * 4u + 0u],
        rotation[i * 4u + 1u],
        rotation[i * 4u + 2u],
        rotation[i * 4u + 3u],
    );
    let half_angle = dt * 0.5;
    let dq = vec4<f32>(0.0, sin(half_angle), 0.0, cos(half_angle));
    let q_new = quat_mul(q, dq);
    rotation[i * 4u + 0u] = q_new.x;
    rotation[i * 4u + 1u] = q_new.y;
    rotation[i * 4u + 2u] = q_new.z;
    rotation[i * 4u + 3u] = q_new.w;
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
    transform_set_position(graphics, Vec3::new(0.0, 6.0, 18.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    let _light =
        light_create_directional(graphics, bevy::color::Color::srgb(0.9, 0.85, 0.8), 300.0)?;

    let cube = geometry_box(0.6, 0.6, 0.6)?;

    let capacity: u32 = 125;
    let mut positions: Vec<f32> = Vec::with_capacity(capacity as usize * 3);
    let mut rotations: Vec<f32> = Vec::with_capacity(capacity as usize * 4);
    let mut scales: Vec<f32> = Vec::with_capacity(capacity as usize * 3);
    for x in 0..5 {
        for y in 0..5 {
            for z in 0..5 {
                positions.push((x as f32 - 2.0) * 1.6);
                positions.push((y as f32 - 2.0) * 1.6);
                positions.push((z as f32 - 2.0) * 1.6);
                // identity quat
                rotations.push(0.0);
                rotations.push(0.0);
                rotations.push(0.0);
                rotations.push(1.0);
                // scale varies per position
                let s = 0.5 + ((x + y + z) as f32 * 0.06);
                scales.push(s);
                scales.push(s);
                scales.push(s);
            }
        }
    }

    let position_attr = geometry_attribute_position();
    let rotation_attr = geometry_attribute_rotation();
    let scale_attr = geometry_attribute_scale();
    let p = particles_create(capacity, vec![position_attr, rotation_attr, scale_attr])?;
    let position_buf = particles_buffer(p, position_attr)?
        .ok_or(error::ProcessingError::ParticlesNotFound)?;
    let rotation_buf = particles_buffer(p, rotation_attr)?
        .ok_or(error::ProcessingError::ParticlesNotFound)?;
    let scale_buf = particles_buffer(p, scale_attr)?
        .ok_or(error::ProcessingError::ParticlesNotFound)?;
    buffer_write(
        position_buf,
        positions.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;
    buffer_write(
        rotation_buf,
        rotations.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;
    buffer_write(
        scale_buf,
        scales.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;

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
            DrawCommand::Particles { particles: p, geometry: cube },
        )?;
        graphics_end_draw(graphics)?;

        compute_set(
            spin,
            "params",
            shader_value::ShaderValue::Float4([0.015, 0.0, 0.0, 0.0]),
        )?;
        particles_apply(p, spin)?;
    }

    Ok(())
}
