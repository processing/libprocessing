use processing_glfw::GlfwContext;
use std::time::Instant;

use bevy::math::Vec3;
use processing::prelude::*;
use processing_render::geometry::AttributeFormat;
use processing_render::render::command::DrawCommand;

const SPAWN_SHADER: &str = r#"
struct Spawn {
    pos: vec4<f32>,
    speed: vec4<f32>,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> color: array<f32>;
@group(0) @binding(3) var<storage, read_write> scale: array<f32>;
@group(0) @binding(4) var<storage, read_write> age: array<f32>;
@group(0) @binding(5) var<storage, read_write> dead: array<f32>;
@group(0) @binding(6) var<uniform> spawn: Spawn;
@group(0) @binding(7) var<uniform> emit_range: vec4<f32>;

fn hash(n: u32) -> u32 {
    var x = n;
    x = (x ^ 61u) ^ (x >> 16u);
    x = x + (x << 3u);
    x = x ^ (x >> 4u);
    x = x * 0x27d4eb2du;
    x = x ^ (x >> 15u);
    return x;
}

fn hash_unit(n: u32) -> f32 {
    return f32(hash(n)) / f32(0xffffffffu);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let local_i = gid.x;
    if local_i >= u32(emit_range.y) { return; }
    let base = u32(emit_range.x);
    let cap  = u32(emit_range.z);
    let slot = (base + local_i) % cap;

    let seed = base + local_i;

    // Random unit-disc direction with some upward bias.
    let theta = hash_unit(seed) * 6.2831853;
    let r     = sqrt(hash_unit(seed * 2u + 1u));
    let dirxz = vec2<f32>(cos(theta), sin(theta)) * r;
    let dy    = 0.7 + 0.3 * hash_unit(seed * 3u + 7u);
    let v     = vec3<f32>(dirxz.x, dy, dirxz.y) * spawn.speed.x;

    position[slot * 3u + 0u] = spawn.pos.x;
    position[slot * 3u + 1u] = spawn.pos.y;
    position[slot * 3u + 2u] = spawn.pos.z;

    velocity[slot * 3u + 0u] = v.x;
    velocity[slot * 3u + 1u] = v.y;
    velocity[slot * 3u + 2u] = v.z;

    let h = fract(hash_unit(seed * 5u + 11u));
    color[slot * 4u + 0u] = 0.5 + 0.5 * sin(h * 6.28);
    color[slot * 4u + 1u] = 0.5 + 0.5 * sin(h * 6.28 + 2.094);
    color[slot * 4u + 2u] = 0.5 + 0.5 * sin(h * 6.28 + 4.189);
    color[slot * 4u + 3u] = 1.0;

    scale[slot * 3u + 0u] = 1.0;
    scale[slot * 3u + 1u] = 1.0;
    scale[slot * 3u + 2u] = 1.0;

    age[slot]  = 0.0;
    dead[slot] = 0.0;
}
"#;

const MOTION_SHADER: &str = r#"
struct Params {
    dt: f32,
    ttl: f32,
    gravity: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> scale: array<f32>;
@group(0) @binding(3) var<storage, read_write> age: array<f32>;
@group(0) @binding(4) var<storage, read_write> dead: array<f32>;
@group(0) @binding(5) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count { return; }
    if dead[i] != 0.0 { return; }

    age[i] = age[i] + params.dt;

    velocity[i * 3u + 1u] = velocity[i * 3u + 1u] - params.gravity * params.dt;

    position[i * 3u + 0u] = position[i * 3u + 0u] + velocity[i * 3u + 0u] * params.dt;
    position[i * 3u + 1u] = position[i * 3u + 1u] + velocity[i * 3u + 1u] * params.dt;
    position[i * 3u + 2u] = position[i * 3u + 2u] + velocity[i * 3u + 2u] * params.dt;

    let life = clamp(1.0 - age[i] / params.ttl, 0.0, 1.0);
    let s = life * life;
    scale[i * 3u + 0u] = s;
    scale[i * 3u + 1u] = s;
    scale[i * 3u + 2u] = s;

    if age[i] > params.ttl { dead[i] = 1.0; }
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
    transform_set_position(graphics, Vec3::new(0.0, 4.0, 16.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 2.0, 0.0))?;

    let _light =
        light_create_directional(graphics, bevy::color::Color::srgb(0.95, 0.9, 0.85), 800.0)?;

    let particle = geometry_sphere(0.12, 8, 6)?;

    let capacity: u32 = 40000;
    let position_attr = geometry_attribute_position();
    let color_attr = geometry_attribute_color();
    let scale_attr = geometry_attribute_scale();
    let dead_attr = geometry_attribute_dead();
    let velocity_attr = geometry_attribute_create("velocity", AttributeFormat::Float3)?;
    let age_attr = geometry_attribute_create("age", AttributeFormat::Float)?;

    let field = field_create(
        capacity,
        vec![
            position_attr,
            color_attr,
            scale_attr,
            dead_attr,
            velocity_attr,
            age_attr,
        ],
    )?;

    // Mark all unemitted slots dead so they don't render at origin.
    let dead_buf = field_pbuffer(field, dead_attr)?
        .ok_or(error::ProcessingError::FieldNotFound)?;
    let init_dead: Vec<u8> = (0..capacity)
        .flat_map(|_| 1.0_f32.to_le_bytes())
        .collect();
    buffer_write(dead_buf, init_dead)?;

    let color_buf = field_pbuffer(field, color_attr)?
        .ok_or(error::ProcessingError::FieldNotFound)?;
    let mat = material_create_field_pbr(color_buf)?;

    let spawn_shader = shader_create(SPAWN_SHADER)?;
    let spawn = compute_create(spawn_shader)?;

    let motion_shader = shader_create(MOTION_SHADER)?;
    let motion = compute_create(motion_shader)?;

    let burst: u32 = 120;
    let dt: f32 = 1.0 / 60.0;
    let ttl: f32 = 2.5;
    let gravity: f32 = 9.8;
    let speed: f32 = 5.0;
    let start = Instant::now();

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
                geometry: particle,
            },
        )?;
        graphics_end_draw(graphics)?;

        // Animate spawn point in a small circle so the fountain meanders.
        let t = start.elapsed().as_secs_f32();
        let sx = t.cos() * 0.4;
        let sz = t.sin() * 0.4;
        compute_set(
            spawn,
            "pos",
            shader_value::ShaderValue::Float4([sx, 7.0, sz, 0.0]),
        )?;
        compute_set(
            spawn,
            "speed",
            shader_value::ShaderValue::Float4([speed, 0.0, 0.0, 0.0]),
        )?;
        field_emit_gpu(field, burst, spawn)?;

        compute_set(motion, "dt", shader_value::ShaderValue::Float(dt))?;
        compute_set(motion, "ttl", shader_value::ShaderValue::Float(ttl))?;
        compute_set(motion, "gravity", shader_value::ShaderValue::Float(gravity))?;
        field_apply(field, motion)?;
    }

    Ok(())
}
