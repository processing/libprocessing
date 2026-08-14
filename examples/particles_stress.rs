//! One million spheres. Still the instancing stress test — tens of millions
//! of triangles a frame, every position rebuilt analytically by a compute
//! kernel each frame — as a sheet of silk.
//!
//! The million pearls are the threads of one wide veil, 1000 x 1000, lifted
//! by three layered traveling waves plus a slow fold that bunches the cloth
//! like wind does. Color is a single continuous pastel gradient woven across
//! the sheet — peach to blush to lavender to sky — so the surface reads as
//! one piece of fabric, never as scattered particles. Flat unlit shading on
//! a warm cream ground.
//!
//! The camera breathes: a log-eased dolly that swoops from a wide view down
//! to skim just above the rippling surface, then lifts back out, drifting
//! around the sheet the whole time.

use std::time::Instant;

use processing_glfw::GlfwContext;

use bevy::math::Vec3;
use processing::prelude::*;
use processing_render::render::command::DrawCommand;

const COLS: u32 = 1000;
const ROWS: u32 = 1000;
const SHEET_W: f32 = 170.0; // world width of the veil
const SHEET_D: f32 = 105.0; // world depth
const AMP: f32 = 9.0; // wave amplitude

const DIVE_PERIOD: f32 = 40.0; // seconds per in-and-out breath
const AZIMUTH_RATE: f32 = 0.06; // radians/sec of orbital drift

const SILK_SHADER: &str = r#"
struct Params {
    time: f32,
    sheet_w: f32,
    sheet_d: f32,
    amp: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
// x = u (0..1 across), y = v (0..1 along), z/w = per-thread jitter
@group(0) @binding(1) var<storage, read> seeds: array<vec4<f32>>;
@group(0) @binding(2) var<uniform> params: Params;

const TAU: f32 = 6.28318530718;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let s = seeds[i];
    let u = s.x;
    let v = s.y;
    let t = params.time;

    // Three traveling waves at different scales and headings make the lift;
    // their sum is the silk. Slow phases: fabric, not water.
    let lift = 0.52 * sin(u * TAU * 1.7 + t * 0.50 + v * 2.3)
             + 0.33 * sin(v * TAU * 1.1 - t * 0.36 + u * 3.1)
             + 0.15 * sin((u + v) * TAU * 3.4 + t * 0.74);

    // A gentle in-plane compression bunches the threads into folds where the
    // cloth crests, the way real fabric gathers.
    let fold_x = 2.6 * sin(v * TAU * 0.9 + t * 0.22);
    let fold_z = 1.8 * sin(u * TAU * 1.3 - t * 0.17);

    let x = (u - 0.5) * params.sheet_w + fold_x + s.z;
    let z = (v - 0.5) * params.sheet_d + fold_z + s.w;
    let y = lift * params.amp;

    position[i * 3u + 0u] = x;
    position[i * 3u + 1u] = y;
    position[i * 3u + 2u] = z;
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

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// One continuous pastel gradient across the sheet: bilinear blend of four
/// corner colors, so neighboring threads always share their hue.
fn silk_color(u: f32, v: f32) -> [f32; 3] {
    const PEACH: [f32; 3] = [1.00, 0.83, 0.72]; // #FFD4B8
    const BLUSH: [f32; 3] = [1.00, 0.68, 0.82]; // #FFAED1
    const LAVENDER: [f32; 3] = [0.80, 0.71, 0.96]; // #CCB5F5
    const SKY: [f32; 3] = [0.74, 0.88, 1.00]; // #BDE0FF
    let top = lerp3(PEACH, BLUSH, u);
    let bottom = lerp3(LAVENDER, SKY, u);
    lerp3(top, bottom, v)
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(900, 700, false)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(900, 700, false)?;
    let graphics = graphics_create(surface, 900, 700, TextureFormat::Rgba16Float)?;

    graphics_mode_3d(graphics)?;

    // Flat pastel rendering: no bloom, no filmic tonemapping, no lights —
    // the material is unlit and the gradient carries everything.
    graphics_remove_bloom(graphics)?;

    let pearl = geometry_sphere(0.16, 6, 4)?;

    let count = COLS * ROWS;
    let position_attr = geometry_attribute_position();
    let color_attr = geometry_attribute_color();
    let p = particles_create(count, vec![position_attr, color_attr])?;

    let mut seeds: Vec<f32> = Vec::with_capacity(count as usize * 4);
    let mut colors: Vec<f32> = Vec::with_capacity(count as usize * 4);

    for i in 0..count {
        let col = i % COLS;
        let row = i / COLS;
        // Sub-thread jitter keeps the weave from moiréing without ever
        // moving a thread far from its neighbors.
        let u = (col as f32 + hash_unit(i ^ 0x517C_C1B7) * 0.9) / COLS as f32;
        let v = (row as f32 + hash_unit(i ^ 0x94D0_49BB) * 0.9) / ROWS as f32;
        let jx = (hash_unit(i ^ 0xC2B2_AE35) - 0.5) * 0.25;
        let jz = (hash_unit(i ^ 0x27D4_EB2F) - 0.5) * 0.25;
        seeds.extend_from_slice(&[u, v, jx, jz]);

        let c = silk_color(u, v);
        colors.extend_from_slice(&[c[0], c[1], c[2], 1.0]);
    }

    let color_buf =
        particles_buffer(p, color_attr)?.ok_or(error::ProcessingError::ParticlesNotFound)?;
    let seed_buf = buffer_create(count as u64 * 4 * 4)?;
    buffer_write(
        color_buf,
        colors.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;
    buffer_write(
        seed_buf,
        seeds.iter().flat_map(|f| f.to_le_bytes()).collect(),
    )?;

    let mat = {
        let m = material_create_unlit()?;
        material_set_albedo_buffer(m, color_buf)?;
        m
    };

    let silk_shader = shader_create(SILK_SHADER)?;
    let silk = compute_create(silk_shader)?;
    compute_set(silk, "seeds", shader_value::ShaderValue::Buffer(seed_buf))?;
    compute_set(silk, "sheet_w", shader_value::ShaderValue::Float(SHEET_W))?;
    compute_set(silk, "sheet_d", shader_value::ShaderValue::Float(SHEET_D))?;
    compute_set(silk, "amp", shader_value::ShaderValue::Float(AMP))?;

    eprintln!("{count} particles");

    let start = Instant::now();
    let far = SHEET_W * 0.95;
    let near = AMP * 2.2; // skimming height at the bottom of the breath

    while glfw_ctx.poll_events() {
        let time = start.elapsed().as_secs_f32();

        // Breathing swoop: wide 3/4 view down to a low glide over the
        // surface and back, log-eased so the descent feels constant-speed,
        // drifting around the sheet all the while.
        let phase = 0.5 - 0.5 * (std::f32::consts::TAU * time / DIVE_PERIOD).cos();
        let dist = (far.ln() + (near.ln() - far.ln()) * phase).exp();
        let az = time * AZIMUTH_RATE;
        let height = AMP * 1.6 + dist * 0.55;
        transform_set_position(
            graphics,
            Vec3::new(az.cos() * dist, height, az.sin() * dist),
        )?;
        // Look slightly past center so the low pass sweeps across the silk
        // instead of pinning to one spot.
        transform_look_at(graphics, Vec3::new(-az.sin() * 8.0, 0.0, az.cos() * 8.0))?;

        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            // Warm cream ground.
            DrawCommand::BackgroundColor(bevy::color::Color::srgb_u8(250, 245, 240)),
        )?;
        graphics_record_command(graphics, DrawCommand::Material(mat))?;
        graphics_record_command(
            graphics,
            DrawCommand::Particles {
                particles: p,
                geometry: Some(pearl),
                topology: processing_render::geometry::Topology::TriangleList,
            },
        )?;
        graphics_end_draw(graphics)?;

        compute_set(silk, "time", shader_value::ShaderValue::Float(time))?;
        particles_apply(p, silk)?;
    }

    Ok(())
}
