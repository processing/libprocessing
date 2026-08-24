use processing_glfw::GlfwContext;
use std::time::Instant;

use bevy::math::Vec3;
use processing::prelude::*;
use processing_render::render::command::{DrawCommand, TextStyle};

const BASE_COUNT: u32 = 6000;
const TRAIL_LEN_MIN: u32 = 8;
const TRAIL_LEN_MAX: u32 = 150;
const TRAIL_LEN: u32 = TRAIL_LEN_MAX;
const CAPACITY: u32 = BASE_COUNT * TRAIL_LEN;
const TRAIL_STRIDE: u32 = 1;

const NOISE_STRENGTH: f32 = 50.5;
const SPHERE_RADIUS: f32 = 1.05;

fn sim_shader() -> String {
    r#"
struct Params {
    base_count: u32,
    trail_len_max: u32,
    trail_len_min: u32,
    seed: u32,
    face_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    noise_scale: f32,
    noise_strength: f32,
    time: f32,
    step_speed: f32,
}

@group(0) @binding(0) var<storage, read>       source_position: array<f32>;
@group(0) @binding(1) var<storage, read>       source_indices:  array<u32>;
@group(0) @binding(2) var<storage, read>       cdf:             array<f32>;
@group(0) @binding(3) var<storage, read_write> head_pos:        array<f32>;
@group(0) @binding(4) var<storage, read_write> anchor_normal:   array<f32>;
@group(0) @binding(5) var<storage, read_write> trail_len_buf:   array<u32>;
@group(0) @binding(6) var<storage, read_write> trail_head_buf:  array<u32>;
@group(0) @binding(7) var<storage, read_write> position:        array<f32>;
@group(0) @binding(8) var<storage, read_write> scale:           array<f32>;
@group(0) @binding(9) var<storage, read_write> life:            array<f32>;
@group(0) @binding(10) var<uniform>            params:          Params;

fn hash_u(n: u32) -> u32 {
    var x = n;
    x = (x ^ 61u) ^ (x >> 16u);
    x = x + (x << 3u);
    x = x ^ (x >> 4u);
    x = x * 0x27d4eb2du;
    x = x ^ (x >> 15u);
    return x;
}
fn hash_unit(n: u32) -> f32 { return f32(hash_u(n)) / f32(0xffffffffu); }

fn cdf_search(u: f32) -> u32 {
    var lo: u32 = 0u;
    var hi: u32 = params.face_count;
    loop {
        if lo >= hi { break; }
        let mid = (lo + hi) >> 1u;
        if cdf[mid] < u { lo = mid + 1u; } else { hi = mid; }
    }
    return min(lo, params.face_count - 1u);
}

fn vhash(p: vec3<f32>) -> f32 {
    let q = fract(p * 0.3183099) + vec3<f32>(0.1, 0.2, 0.3);
    let r = q + dot(q, q.yzx + 19.19);
    return fract(r.x * r.y * r.z);
}
fn value_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(
            mix(vhash(i + vec3<f32>(0.0, 0.0, 0.0)), vhash(i + vec3<f32>(1.0, 0.0, 0.0)), u.x),
            mix(vhash(i + vec3<f32>(0.0, 1.0, 0.0)), vhash(i + vec3<f32>(1.0, 1.0, 0.0)), u.x),
            u.y),
        mix(
            mix(vhash(i + vec3<f32>(0.0, 0.0, 1.0)), vhash(i + vec3<f32>(1.0, 0.0, 1.0)), u.x),
            mix(vhash(i + vec3<f32>(0.0, 1.0, 1.0)), vhash(i + vec3<f32>(1.0, 1.0, 1.0)), u.x),
            u.y),
        u.z);
}
fn noise3(p: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        value_noise(p),
        value_noise(p + vec3<f32>(31.4, 0.0, 0.0)),
        value_noise(p + vec3<f32>(0.0, 71.7, 0.0)),
    ) * 2.0 - 1.0;
}
fn curl_noise(p: vec3<f32>) -> vec3<f32> {
    let eps = 0.01;
    let dx = vec3<f32>(eps, 0.0, 0.0);
    let dy = vec3<f32>(0.0, eps, 0.0);
    let dz = vec3<f32>(0.0, 0.0, eps);
    let n_xp = noise3(p + dx); let n_xm = noise3(p - dx);
    let n_yp = noise3(p + dy); let n_ym = noise3(p - dy);
    let n_zp = noise3(p + dz); let n_zm = noise3(p - dz);
    let inv = 1.0 / (2.0 * eps);
    let dn_dx = (n_xp - n_xm) * inv;
    let dn_dy = (n_yp - n_ym) * inv;
    let dn_dz = (n_zp - n_zm) * inv;
    return vec3<f32>(
        dn_dy.z - dn_dz.y,
        dn_dz.x - dn_dx.z,
        dn_dx.y - dn_dy.x,
    );
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= params.base_count { return; }

    let prev_len = trail_len_buf[i];
    let prev_head = trail_head_buf[i];
    let next_head_if_drift =
        select((prev_head + 1u) % prev_len, 0u, prev_len == 0u);
    let do_reset = (prev_len == 0u) || (next_head_if_drift == 0u);

    var hp: vec3<f32>;
    var an: vec3<f32>;
    var head_slot: u32 = 0u;
    var len_i: u32 = prev_len;

    if do_reset {
        let cycle_seed = params.seed ^ (i * 2654435761u + prev_head * 7919u + 1u);
        let u01 = hash_unit(cycle_seed * 7u + 13u);
        let face = cdf_search(u01);
        let i0 = source_indices[face * 3u + 0u];
        let i1 = source_indices[face * 3u + 1u];
        let i2 = source_indices[face * 3u + 2u];
        let p0 = vec3<f32>(source_position[i0 * 3u + 0u], source_position[i0 * 3u + 1u], source_position[i0 * 3u + 2u]);
        let p1 = vec3<f32>(source_position[i1 * 3u + 0u], source_position[i1 * 3u + 1u], source_position[i1 * 3u + 2u]);
        let p2 = vec3<f32>(source_position[i2 * 3u + 0u], source_position[i2 * 3u + 1u], source_position[i2 * 3u + 2u]);
        var u = hash_unit(cycle_seed * 31u + 23u);
        var v = hash_unit(cycle_seed * 47u + 29u);
        if u + v > 1.0 { u = 1.0 - u; v = 1.0 - v; }
        hp = (1.0 - u - v) * p0 + u * p1 + v * p2;
        an = normalize(cross(p1 - p0, p2 - p0));

        let r = hash_unit(cycle_seed * 0x9E3779B9u + 17u);
        let r_biased = pow(r, 5.0);
        let span = f32(params.trail_len_max - params.trail_len_min);
        len_i = params.trail_len_min + u32(r_biased * span);
        if len_i < params.trail_len_min { len_i = params.trail_len_min; }
        if len_i > params.trail_len_max { len_i = params.trail_len_max; }
        head_slot = 0u;

        head_pos[i * 3u + 0u] = hp.x;
        head_pos[i * 3u + 1u] = hp.y;
        head_pos[i * 3u + 2u] = hp.z;
        anchor_normal[i * 3u + 0u] = an.x;
        anchor_normal[i * 3u + 1u] = an.y;
        anchor_normal[i * 3u + 2u] = an.z;
        trail_len_buf[i] = len_i;
        trail_head_buf[i] = head_slot;

        let base = i * params.trail_len_max;
        for (var k = 0u; k < params.trail_len_max; k = k + 1u) {
            let s = base + k;
            position[s * 3u + 0u] = hp.x;
            position[s * 3u + 1u] = hp.y;
            position[s * 3u + 2u] = hp.z;
            scale[s * 3u + 0u] = 1.0;
            scale[s * 3u + 1u] = 1.0;
            scale[s * 3u + 2u] = 1.0;
            if k < len_i {
                life[s] = 1.0;
            } else {
                life[s] = 0.0;
            }
        }
    } else {
        hp = vec3<f32>(
            head_pos[i * 3u + 0u],
            head_pos[i * 3u + 1u],
            head_pos[i * 3u + 2u],
        );
        an = vec3<f32>(
            anchor_normal[i * 3u + 0u],
            anchor_normal[i * 3u + 1u],
            anchor_normal[i * 3u + 2u],
        );
        let sample = hp * params.noise_scale
            + vec3<f32>(params.time, params.time * 0.7, params.time * 1.3);
        let raw = curl_noise(sample);
        let tangent = raw - dot(raw, an) * an;
        hp = hp + tangent * params.noise_strength * params.step_speed;
        head_pos[i * 3u + 0u] = hp.x;
        head_pos[i * 3u + 1u] = hp.y;
        head_pos[i * 3u + 2u] = hp.z;
        head_slot = next_head_if_drift;
        trail_head_buf[i] = head_slot;

        let trail_slot = i * params.trail_len_max + head_slot;
        position[trail_slot * 3u + 0u] = hp.x;
        position[trail_slot * 3u + 1u] = hp.y;
        position[trail_slot * 3u + 2u] = hp.z;
        scale[trail_slot * 3u + 0u] = 1.0;
        scale[trail_slot * 3u + 1u] = 1.0;
        scale[trail_slot * 3u + 2u] = 1.0;
        life[trail_slot] = 1.0;
    }
}
"#
    .to_string()
}

const COLOR_FADE: &str = r#"
struct Params {
    base_count: u32,
    trail_len_max: u32,
}
@group(0) @binding(0) var<storage, read_write> color:          array<f32>;
@group(0) @binding(1) var<storage, read>       trail_len_buf:  array<u32>;
@group(0) @binding(2) var<storage, read>       trail_head_buf: array<u32>;
@group(0) @binding(3) var<uniform>             params:         Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let s = gid.x;
    let total = params.base_count * params.trail_len_max;
    if s >= total { return; }
    let i = s / params.trail_len_max;
    let k = s % params.trail_len_max;

    let len_i = trail_len_buf[i];
    if len_i == 0u || k >= len_i {
        return;
    }

    let head = trail_head_buf[i];
    var slot_age: u32 = 0u;
    if k <= head {
        slot_age = head - k;
    } else {
        slot_age = head + len_i - k;
    }
    let denom = max(len_i - 1u, 1u);
    let t = f32(slot_age) / f32(denom);

    let head_c = vec3<f32>(0.2, 0.8, 1.0) * 25.0;
    let tail_c = vec3<f32>(1.0, 0.35, 0.45) * 55.0;
    let c = mix(head_c, tail_c, t);
    color[s * 4u + 0u] = c.x;
    color[s * 4u + 1u] = c.y;
    color[s * 4u + 2u] = c.z;
    color[s * 4u + 3u] = 1.0;
}
"#;

fn main() {
    sketch().unwrap();
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(1200, 800, false)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(1200, 800, false)?;
    let graphics = graphics_create(surface, 1200, 800, TextureFormat::Rgba16Float)?;

    graphics_mode_3d(graphics)?;
    transform_set_position(graphics, Vec3::new(0.0, 0.0, 1700.0))?;
    transform_look_at(graphics, Vec3::ZERO)?;
    graphics_orbit_camera(graphics)?;

    const TEXT_PT: f32 = 700.0;
    const EXTRUSION: f32 = 70.0;
    processing_core::app_mut(|app| {
        let mut state = app
            .world_mut()
            .get_mut::<processing_render::render::RenderState>(graphics)
            .ok_or(error::ProcessingError::GraphicsNotFound)?;
        state.style.text_size = TEXT_PT;
        Ok(())
    })?;
    graphics_record_command(graphics, DrawCommand::TextStyle(TextStyle::Bold))?;

    let text = "processing5";
    let w = graphics_text_width(graphics, text)?;
    println!("text width = {w} (size = {TEXT_PT}, extrusion = {EXTRUSION})");
    let mesh = graphics_text_to_model(graphics, text, -w / 2.0, -TEXT_PT * 0.4, EXTRUSION)?;
    let source = geometry_create_from_mesh(mesh)?;

    let (cdf_bytes, indices_bytes, face_count) = processing_core::app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(processing_render::particles::prepare_scatter_source, source)
            .unwrap()
    })?;
    let cdf_buf = buffer_create_with_data(cdf_bytes)?;
    let idx_buf = buffer_create_with_data(indices_bytes)?;

    let head_pos_buf = buffer_create(BASE_COUNT as u64 * 3 * 4)?;
    let anchor_normal_buf = buffer_create(BASE_COUNT as u64 * 3 * 4)?;
    let trail_len_buf = buffer_create(BASE_COUNT as u64 * 4)?;
    let trail_head_buf = buffer_create(BASE_COUNT as u64 * 4)?;

    let particle = geometry_sphere(SPHERE_RADIUS, 5, 4)?;

    let p = particles_create(
        CAPACITY,
        vec![
            geometry_attribute_position(),
            geometry_attribute_scale(),
            geometry_attribute_life(),
            geometry_attribute_color(),
        ],
    )?;

    let sim_src = sim_shader();
    let sim_shader_e = shader_create(&sim_src)?;
    let sim = compute_create(sim_shader_e)?;
    compute_set(
        sim,
        "source_position",
        shader_value::ShaderValue::MeshAttribute(source, geometry_attribute_position()),
    )?;
    compute_set(
        sim,
        "source_indices",
        shader_value::ShaderValue::Buffer(idx_buf),
    )?;
    compute_set(sim, "cdf", shader_value::ShaderValue::Buffer(cdf_buf))?;
    compute_set(
        sim,
        "head_pos",
        shader_value::ShaderValue::Buffer(head_pos_buf),
    )?;
    compute_set(
        sim,
        "anchor_normal",
        shader_value::ShaderValue::Buffer(anchor_normal_buf),
    )?;
    compute_set(
        sim,
        "trail_len_buf",
        shader_value::ShaderValue::Buffer(trail_len_buf),
    )?;
    compute_set(
        sim,
        "trail_head_buf",
        shader_value::ShaderValue::Buffer(trail_head_buf),
    )?;
    compute_set(
        sim,
        "base_count",
        shader_value::ShaderValue::UInt(BASE_COUNT),
    )?;
    compute_set(
        sim,
        "trail_len_max",
        shader_value::ShaderValue::UInt(TRAIL_LEN_MAX),
    )?;
    compute_set(
        sim,
        "trail_len_min",
        shader_value::ShaderValue::UInt(TRAIL_LEN_MIN),
    )?;
    compute_set(
        sim,
        "face_count",
        shader_value::ShaderValue::UInt(face_count),
    )?;
    compute_set(sim, "seed", shader_value::ShaderValue::UInt(0xc0ffeeu32))?;
    compute_set(sim, "noise_scale", shader_value::ShaderValue::Float(0.005))?;
    compute_set(
        sim,
        "noise_strength",
        shader_value::ShaderValue::Float(NOISE_STRENGTH),
    )?;
    compute_set(sim, "step_speed", shader_value::ShaderValue::Float(0.05))?;

    compute_set(sim, "time", shader_value::ShaderValue::Float(0.0))?;
    particles_apply(p, sim)?;

    let color_buf = particles_buffer(p, geometry_attribute_color())?
        .ok_or(error::ProcessingError::ParticlesNotFound)?;
    let mat = material_create_unlit()?;
    material_set_albedo_buffer(mat, color_buf)?;

    let fade_shader_e = shader_create(COLOR_FADE)?;
    let fade = compute_create(fade_shader_e)?;
    compute_set(
        fade,
        "base_count",
        shader_value::ShaderValue::UInt(BASE_COUNT),
    )?;
    compute_set(
        fade,
        "trail_len_max",
        shader_value::ShaderValue::UInt(TRAIL_LEN_MAX),
    )?;
    compute_set(
        fade,
        "trail_len_buf",
        shader_value::ShaderValue::Buffer(trail_len_buf),
    )?;
    compute_set(
        fade,
        "trail_head_buf",
        shader_value::ShaderValue::Buffer(trail_head_buf),
    )?;

    particles_apply(p, fade)?;

    let start = Instant::now();
    let mut render_frame: u32 = 0;
    while glfw_ctx.poll_events() {
        if render_frame % TRAIL_STRIDE == 0 {
            let t = start.elapsed().as_secs_f32();
            compute_set(sim, "time", shader_value::ShaderValue::Float(t * 0.015))?;
            particles_apply(p, sim)?;
            particles_apply(p, fade)?;
        }
        render_frame = render_frame.wrapping_add(1);

        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.02, 0.02, 0.04)),
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
    }

    Ok(())
}
