use processing_glfw::GlfwContext;

use bevy::math::Vec3;
use processing::prelude::*;
use processing_render::geometry::AttributeFormat;
use processing_render::render::command::DrawCommand;

const AGING_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read_write> age: array<f32>;
@group(0) @binding(1) var<storage, read_write> dead: array<f32>;
@group(0) @binding(2) var<storage, read_write> position: array<f32>;
@group(0) @binding(3) var<storage, read_write> scale: array<f32>;
@group(0) @binding(4) var<uniform> params: vec4<f32>;  // x = dt, y = ttl

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count {
        return;
    }
    let dt = params.x;
    let ttl = params.y;

    if dead[i] != 0.0 {
        return;
    }

    age[i] = age[i] + dt;
    // gravity-ish drop
    position[i * 3u + 1u] = position[i * 3u + 1u] - dt * 1.5;

    // Shrink toward zero as age approaches ttl so dying is visible.
    let life = clamp(1.0 - age[i] / ttl, 0.0, 1.0);
    let s = life * life;  // ease out
    scale[i * 3u + 0u] = s;
    scale[i * 3u + 1u] = s;
    scale[i * 3u + 2u] = s;

    if age[i] > ttl {
        dead[i] = 1.0;
    }
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
    transform_set_position(graphics, Vec3::new(0.0, 2.0, 14.0))?;
    transform_look_at(graphics, Vec3::new(0.0, 0.0, 0.0))?;

    let sphere = geometry_sphere(0.1, 8, 6)?;

    let capacity: u32 = 800;
    let position_attr = geometry_attribute_position();
    let color_attr = geometry_attribute_color();
    let scale_attr = geometry_attribute_scale();
    let dead_attr = geometry_attribute_dead();
    let age_attr = geometry_attribute_create("age", AttributeFormat::Float)?;

    let p = particles_create(
        capacity,
        vec![
            position_attr,
            color_attr,
            scale_attr,
            dead_attr,
            age_attr,
        ],
    )?;
    let dead_buf = particles_buffer(p, dead_attr)?
        .ok_or(error::ProcessingError::ParticlesNotFound)?;
    let color_buf = particles_buffer(p, color_attr)?
        .ok_or(error::ProcessingError::ParticlesNotFound)?;

    // Mark all slots dead initially so the unemitted ring slots don't render.
    let init_dead: Vec<u8> = (0..capacity)
        .flat_map(|_| 1.0_f32.to_le_bytes())
        .collect();
    buffer_write(dead_buf, init_dead)?;

    let mat = { let m = material_create_unlit()?; material_set_albedo_buffer(m, color_buf)?; m };
    let aging_shader = shader_create(AGING_SHADER)?;
    let aging = compute_create(aging_shader)?;

    let burst: u32 = 6;
    let dt: f32 = 1.0 / 60.0;
    let ttl: f32 = 1.0;
    let mut frame: u32 = 0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.04, 0.04, 0.07)),
        )?;
        graphics_record_command(graphics, DrawCommand::Material(mat))?;
        graphics_record_command(
            graphics,
            DrawCommand::Particles { particles: p, geometry: sphere },
        )?;
        graphics_end_draw(graphics)?;

        let mut positions: Vec<f32> = Vec::with_capacity(burst as usize * 3);
        let mut colors: Vec<f32> = Vec::with_capacity(burst as usize * 4);
        for k in 0..burst {
            let i = frame * burst + k;
            let u = ((i.wrapping_mul(2654435761) >> 8) & 0xFFFF) as f32 / 65535.0;
            let v = ((i.wrapping_mul(40503) >> 8) & 0xFFFF) as f32 / 65535.0;
            let theta = u * std::f32::consts::TAU;
            let r = v * 0.6;
            positions.push(theta.cos() * r);
            positions.push(2.5);
            positions.push(theta.sin() * r);
            let h = (i as f32 * 0.013) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb(h, 0.85, 1.0);
            colors.push(cr);
            colors.push(cg);
            colors.push(cb);
            colors.push(1.0);
        }
        let position_bytes: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        let color_bytes: Vec<u8> = colors.iter().flat_map(|f| f.to_le_bytes()).collect();
        let zero_floats: Vec<u8> = (0..burst).flat_map(|_| 0.0_f32.to_le_bytes()).collect();
        // init scale to 1; the aging shader shrinks it over time
        let one_scale: Vec<u8> = (0..burst)
            .flat_map(|_| {
                [1.0_f32, 1.0, 1.0]
                    .iter()
                    .flat_map(|f| f.to_le_bytes())
                    .collect::<Vec<u8>>()
            })
            .collect();
        particles_emit(
            p,
            burst,
            vec![
                (position_attr, position_bytes),
                (color_attr, color_bytes),
                (scale_attr, one_scale),
                (age_attr, zero_floats.clone()),
                (dead_attr, zero_floats),
            ],
        )?;

        compute_set(
            aging,
            "params",
            shader_value::ShaderValue::Float4([dt, ttl, 0.0, 0.0]),
        )?;
        particles_apply(p, aging)?;

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
