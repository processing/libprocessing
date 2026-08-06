use processing_glfw::GlfwContext;
use std::time::Instant;

use bevy::math::Vec3;
use processing::prelude::*;
use processing_render::geometry::AttributeFormat;
use processing_render::render::command::DrawCommand;

const AGE_SHADER: &str = r#"
struct Params { dt: f32, ttl: f32, _pad0: f32, _pad1: f32 }

@group(0) @binding(0) var<storage, read_write> scale: array<f32>;
@group(0) @binding(1) var<storage, read_write> age:   array<f32>;
@group(0) @binding(2) var<storage, read_write> life:  array<f32>;
@group(0) @binding(3) var<uniform>             params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count { return; }
    if life[i] <= 0.0 { return; }

    age[i] = age[i] + params.dt;
    let t = age[i] / params.ttl;
    let rise = clamp(t / 0.05, 0.0, 1.0);
    let fall = clamp((1.0 - t) / 0.6, 0.0, 1.0);
    let s = rise * fall;
    scale[i * 3u + 0u] = s;
    scale[i * 3u + 1u] = s;
    scale[i * 3u + 2u] = s;

    if age[i] > params.ttl { life[i] = 0.0; }
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
    transform_set_position(graphics, Vec3::new(0.0, 0.4, 4.5))?;
    transform_look_at(graphics, Vec3::ZERO)?;

    let _key = light_create_directional(
        graphics,
        bevy::color::Color::srgb(1.0, 0.95, 0.85),
        4500.0,
    )?;

    let source = geometry_sphere(1.2, 96, 48)?;
    let scatter = particles_scatter_create(source)?;

    let particle = geometry_sphere(0.005, 6, 4)?;

    let capacity: u32 = 40_000;
    let position_attr = geometry_attribute_position();
    let scale_attr = geometry_attribute_scale();
    let life_attr = geometry_attribute_life();
    let age_attr = geometry_attribute_create("age", AttributeFormat::Float)?;

    let p = particles_create(
        capacity,
        vec![position_attr, scale_attr, life_attr, age_attr],
    )?;

    let age_shader = shader_create(AGE_SHADER)?;
    let aging = compute_create(age_shader)?;

    let mat = material_create_pbr()?;
    material_set_albedo_color(mat, [0.9, 0.85, 1.0, 1.0])?;

    let burst: u32 = 600;
    let dt: f32 = 1.0 / 60.0;
    let ttl: f32 = 4.0;
    let start = Instant::now();

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.03, 0.03, 0.05)),
        )?;
        graphics_record_command(graphics, DrawCommand::Material(mat))?;
        graphics_record_command(
            graphics,
            DrawCommand::Particles {
                particles: p,
                geometry: particle,
            },
        )?;
        graphics_end_draw(graphics)?;

        let t = start.elapsed().as_secs_f32();
        let cam_x = (t * 0.25).cos() * 4.5;
        let cam_z = (t * 0.25).sin() * 4.5;
        transform_set_position(graphics, Vec3::new(cam_x, 0.4, cam_z))?;
        transform_look_at(graphics, Vec3::ZERO)?;

        compute_set(
            scatter,
            "seed",
            shader_value::ShaderValue::UInt((t * 1000.0) as u32 ^ 0xc0ffeeu32),
        )?;
        particles_emit_gpu(p, burst, scatter)?;

        compute_set(aging, "dt", shader_value::ShaderValue::Float(dt))?;
        compute_set(aging, "ttl", shader_value::ShaderValue::Float(ttl))?;
        particles_apply(p, aging)?;
    }

    Ok(())
}
