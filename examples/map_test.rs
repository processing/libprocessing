//! `map`: in-place and out-of-place.

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

    let n_particles = 5u32;
    let components = 3u32;
    let input: Vec<f32> = (0..(n_particles * components)).map(|i| i as f32).collect();

    let mut ok = true;

    let a = buffer_create_with_data(to_bytes(&input))?;
    map(a, a, components, MAP_AFFINE, 2.0, 1.0)?;
    let got = f32s(&buffer_read(a)?);
    let want: Vec<f32> = input.iter().map(|x| x * 2.0 + 1.0).collect();
    if approx(&got, &want) {
        println!(
            "  PASS map in-place, all {} components: {:?}",
            got.len(),
            got
        );
    } else {
        ok = false;
        println!("  FAIL map in-place: got {got:?}, want {want:?}");
    }
    buffer_destroy(a)?;

    let neg: Vec<f32> = (0..(n_particles * components))
        .map(|i| -(i as f32) - 0.5)
        .collect();
    let a2 = buffer_create_with_data(to_bytes(&neg))?;
    let dst = buffer_create_with_data(to_bytes(&vec![0.0; neg.len()]))?;
    map(dst, a2, components, MAP_ABS, 0.0, 0.0)?;
    let got_dst = f32s(&buffer_read(dst)?);
    let got_a = f32s(&buffer_read(a2)?);
    let want_dst: Vec<f32> = neg.iter().map(|x| x.abs()).collect();
    if approx(&got_dst, &want_dst) && approx(&got_a, &neg) {
        println!("  PASS map out-of-place: dst={got_dst:?}, a preserved");
    } else {
        ok = false;
        println!(
            "  FAIL map out-of-place: dst={got_dst:?} (want {want_dst:?}), a={got_a:?} (want {neg:?})"
        );
    }
    buffer_destroy(a2)?;
    buffer_destroy(dst)?;

    Ok(ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("map_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("map_test: FAILURES");
        exit(1).unwrap();
    }
}
