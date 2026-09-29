//! Shared `falloff` WESL helper against CPU falloff at known distances.

use bevy::prelude::Entity;
use processing::prelude::*;

const RADIUS: f32 = 2.0;

fn f32s(b: &[u8]) -> Vec<f32> {
    b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
fn to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}
fn approx(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5)
}

const DISTS: [f32; 4] = [0.0, 0.5, 1.0, 1.5];

fn cpu_falloff(d: f32, mode: u32) -> f32 {
    let n = 1.0 - d / RADIUS;
    match mode {
        1 => n,
        2 => n * n * (3.0 - 2.0 * n),
        3 => n * n,
        4 => n * n * n,
        5 => RADIUS / (d + RADIUS),
        _ => 1.0,
    }
}

fn run_mode(field: Entity, weight: Entity, mode: u32) -> error::Result<bool> {
    compute_set(field, "falloff_mode", shader_value::ShaderValue::UInt(mode))?;
    compute_dispatch(field, 1, 1, 1)?;
    let got = f32s(&buffer_read(weight)?);
    let want: Vec<f32> = DISTS.iter().map(|&d| cpu_falloff(d, mode)).collect();
    if approx(&got, &want) {
        println!("  PASS falloff mode {mode}: {got:?}");
        Ok(true)
    } else {
        println!("  FAIL falloff mode {mode}: got {got:?}, want {want:?}");
        Ok(false)
    }
}

fn sketch() -> error::Result<bool> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    let positions: Vec<f32> = DISTS.iter().flat_map(|&d| [d, 0.0, 0.0]).collect();
    let pos = buffer_create_with_data(to_bytes(&positions))?;
    let weight = buffer_create_with_data(to_bytes(&vec![0.0; DISTS.len()]))?;

    let field = particles_kernel_field()?;
    compute_set(field, "position", shader_value::ShaderValue::Buffer(pos))?;
    compute_set(field, "weight", shader_value::ShaderValue::Buffer(weight))?;
    compute_set(
        field,
        "center",
        shader_value::ShaderValue::Float3([0.0, 0.0, 0.0]),
    )?;
    compute_set(field, "radius", shader_value::ShaderValue::Float(RADIUS))?;

    let mut ok = true;
    for mode in [0u32, 1, 2, 3, 4, 5] {
        ok &= run_mode(field, weight, mode)?;
    }

    // the other importers must compile too
    for (name, r) in [
        ("attract", particles_kernel_attract()),
        ("vortex", particles_kernel_vortex()),
        ("impulse", particles_kernel_impulse()),
    ] {
        match r {
            Ok(_) => println!("  PASS {name} compiles (falloff import)"),
            Err(e) => {
                ok = false;
                println!("  FAIL {name} compile: {e}");
            }
        }
    }
    Ok(ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("field_falloff_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("field_falloff_test: FAILURES");
        exit(1).unwrap();
    }
}
