//! `combine` and `mix`, including the alias guard.

use processing::prelude::*;

fn f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
fn to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}
fn approx(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5)
}

fn sketch() -> error::Result<bool> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    let mut ok = true;

    let av: Vec<f32> = (0..12).map(|i| i as f32).collect();
    let bv: Vec<f32> = (0..12).map(|i| (i * 10) as f32).collect();
    let a = buffer_create_with_data(to_bytes(&av))?;
    let b = buffer_create_with_data(to_bytes(&bv))?;
    combine(a, a, b, 3, COMBINE_ADD, 1.0, 0.0)?;
    let got = f32s(&buffer_read(a)?);
    let want: Vec<f32> = av.iter().zip(&bv).map(|(x, y)| x + y).collect();
    if approx(&got, &want) {
        println!("  PASS combine in-place a+=b");
    } else {
        ok = false;
        println!("  FAIL combine in-place: got {got:?}, want {want:?}");
    }

    let dst = buffer_create_with_data(to_bytes(&vec![0.0; 12]))?;
    combine(dst, a, b, 3, COMBINE_MUL, 1.0, 0.0)?;
    let got_dst = f32s(&buffer_read(dst)?);
    let cur_a = f32s(&buffer_read(a)?);
    let want_dst: Vec<f32> = want.iter().zip(&bv).map(|(x, y)| x * y).collect();
    if approx(&got_dst, &want_dst) && approx(&cur_a, &want) {
        println!("  PASS combine out-of-place dst=a*b (inputs preserved)");
    } else {
        ok = false;
        println!("  FAIL combine out-of-place: dst={got_dst:?} want {want_dst:?}");
    }

    match combine(b, a, b, 3, COMBINE_ADD, 1.0, 0.0) {
        Err(_) => println!("  PASS combine alias guard (dst==b rejected before dispatch)"),
        Ok(()) => {
            ok = false;
            println!("  FAIL combine alias guard: expected Err, got Ok");
        }
    }

    buffer_destroy(a)?;
    buffer_destroy(b)?;
    buffer_destroy(dst)?;

    let comps = 4u32;
    let a2: Vec<f32> = vec![1.0; 12];
    let b2: Vec<f32> = vec![3.0; 12];
    let tv = vec![0.0f32, 0.5, 1.0];
    let am = buffer_create_with_data(to_bytes(&a2))?;
    let bm = buffer_create_with_data(to_bytes(&b2))?;
    let tm = buffer_create_with_data(to_bytes(&tv))?;
    let dm = buffer_create_with_data(to_bytes(&vec![0.0; 12]))?;
    mix(dm, am, bm, tm, comps, 1.0, 0.0, true)?;
    let got_mix = f32s(&buffer_read(dm)?);
    let mut want_mix = vec![1.0f32; 4];
    want_mix.extend([2.0f32; 4]);
    want_mix.extend([3.0f32; 4]);
    if approx(&got_mix, &want_mix) {
        println!("  PASS mix out-of-place (per-particle t broadcast across components)");
    } else {
        ok = false;
        println!("  FAIL mix out-of-place: got {got_mix:?}, want {want_mix:?}");
    }

    mix(am, am, bm, tm, comps, 1.0, 0.0, true)?;
    let got_mix_ip = f32s(&buffer_read(am)?);
    if approx(&got_mix_ip, &want_mix) {
        println!("  PASS mix in-place");
    } else {
        ok = false;
        println!("  FAIL mix in-place: got {got_mix_ip:?}, want {want_mix:?}");
    }

    buffer_destroy(am)?;
    buffer_destroy(bm)?;
    buffer_destroy(tm)?;
    buffer_destroy(dm)?;

    Ok(ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("combine_mix_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("combine_mix_test: FAILURES");
        exit(1).unwrap();
    }
}
