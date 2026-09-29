//! GPU->CPU `reduce` (sum / min / max), single- and multi-level.

use bevy::prelude::Entity;
use processing::prelude::*;

fn to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn check(label: &str, got: f32, want: f32) -> bool {
    if (got - want).abs() <= 1e-3 * want.abs().max(1.0) {
        println!("  PASS {label}: {got}");
        true
    } else {
        println!("  FAIL {label}: got {got}, want {want}");
        false
    }
}

fn reduce_all(vals: &[f32]) -> error::Result<(f32, f32, f32)> {
    let b = buffer_create_with_data(to_bytes(vals))?;
    let sum = reduce(b, REDUCE_OP_SUM)?;
    let mn = reduce(b, REDUCE_OP_MIN)?;
    let mx = reduce(b, REDUCE_OP_MAX)?;
    let _: Entity = b;
    buffer_destroy(b)?;
    Ok((sum, mn, mx))
}

fn sketch() -> error::Result<bool> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    let mut ok = true;

    let (s, mn, mx) = reduce_all(&vec![1.0; 1000])?;
    ok &= check("ones/1000 sum", s, 1000.0);
    ok &= check("ones/1000 min", mn, 1.0);
    ok &= check("ones/1000 max", mx, 1.0);

    let ramp: Vec<f32> = (0..300).map(|i| i as f32).collect();
    let (s, mn, mx) = reduce_all(&ramp)?;
    ok &= check("ramp/300 sum", s, 300.0 * 299.0 / 2.0);
    ok &= check("ramp/300 min", mn, 0.0);
    ok &= check("ramp/300 max", mx, 299.0);

    let (s, mn, mx) = reduce_all(&vec![1.0; 70_000])?;
    ok &= check("ones/70k sum", s, 70_000.0);
    ok &= check("ones/70k min", mn, 1.0);
    ok &= check("ones/70k max", mx, 1.0);

    let big: Vec<f32> = (0..70_000).map(|i| i as f32).collect();
    let b = buffer_create_with_data(to_bytes(&big))?;
    ok &= check("ramp/70k min", reduce(b, REDUCE_OP_MIN)?, 0.0);
    ok &= check("ramp/70k max", reduce(b, REDUCE_OP_MAX)?, 69_999.0);
    buffer_destroy(b)?;

    Ok(ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("reduce_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("reduce_test: FAILURES");
        exit(1).unwrap();
    }
}
