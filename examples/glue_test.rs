//! Glue algebra verbs: `reduce_components`, `extract`, `pack`, `generate`.

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
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
}

fn sketch() -> error::Result<bool> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    let mut ok = true;

    let vel = vec![
        3.0f32, 4.0, 0.0, 1.0, 2.0, 2.0, 0.0, 0.0, 7.0, 5.0, 0.0, 0.0,
    ];

    let v = buffer_create_with_data(to_bytes(&vel))?;
    let speed = buffer_create_with_data(to_bytes(&vec![0.0; 4]))?;
    reduce_components(speed, v, 3, REDUCE_LENGTH)?;
    let got = f32s(&buffer_read(speed)?);
    if approx(&got, &[5.0, 3.0, 7.0, 5.0]) {
        println!("  PASS reduce_components LENGTH (speed=|velocity|): {got:?}");
    } else {
        ok = false;
        println!("  FAIL reduce LENGTH: got {got:?}, want [5,3,7,5]");
    }

    let y = buffer_create_with_data(to_bytes(&vec![0.0; 4]))?;
    extract(y, v, 3, 1)?;
    let got_y = f32s(&buffer_read(y)?);
    if approx(&got_y, &[4.0, 2.0, 0.0, 0.0]) {
        println!("  PASS extract .y: {got_y:?}");
    } else {
        ok = false;
        println!("  FAIL extract: got {got_y:?}, want [4,2,0,0]");
    }
    buffer_destroy(v)?;
    buffer_destroy(speed)?;
    buffer_destroy(y)?;

    let xs = buffer_create_with_data(to_bytes(&[1.0, 2.0, 3.0]))?;
    let ys = buffer_create_with_data(to_bytes(&[10.0, 20.0, 30.0]))?;
    let zs = buffer_create_with_data(to_bytes(&[100.0, 200.0, 300.0]))?;
    let packed = buffer_create_with_data(to_bytes(&vec![0.0; 9]))?;
    pack(packed, &[xs, ys, zs])?;
    let got_p = f32s(&buffer_read(packed)?);
    if approx(
        &got_p,
        &[1.0, 10.0, 100.0, 2.0, 20.0, 200.0, 3.0, 30.0, 300.0],
    ) {
        println!("  PASS pack x/y/z -> Float3: {got_p:?}");
    } else {
        ok = false;
        println!("  FAIL pack: got {got_p:?}");
    }
    buffer_destroy(xs)?;
    buffer_destroy(ys)?;
    buffer_destroy(zs)?;
    buffer_destroy(packed)?;

    let g1 = buffer_create_with_data(to_bytes(&vec![0.0; 16]))?;
    let g2 = buffer_create_with_data(to_bytes(&vec![0.0; 16]))?;
    let g3 = buffer_create_with_data(to_bytes(&vec![0.0; 16]))?;
    generate(g1, 2, GEN_UNIFORM, 42, 1.0, 0.0)?;
    generate(g2, 2, GEN_UNIFORM, 42, 1.0, 0.0)?; // same seed
    generate(g3, 2, GEN_UNIFORM, 43, 1.0, 0.0)?; // different seed
    let a = f32s(&buffer_read(g1)?);
    let b = f32s(&buffer_read(g2)?);
    let c = f32s(&buffer_read(g3)?);
    let in_range = a.iter().all(|x| (0.0..1.0).contains(x));
    let deterministic = approx(&a, &b);
    let seed_varies = !approx(&a, &c);
    let varied = a.windows(2).any(|w| (w[0] - w[1]).abs() > 1e-6);
    if in_range && deterministic && seed_varies && varied {
        println!("  PASS generate uniform (in-range, deterministic, seed-sensitive)");
    } else {
        ok = false;
        println!(
            "  FAIL generate: in_range={in_range}, deterministic={deterministic}, seed_varies={seed_varies}, varied={varied}"
        );
    }
    buffer_destroy(g1)?;
    buffer_destroy(g2)?;
    buffer_destroy(g3)?;

    Ok(ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("glue_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("glue_test: FAILURES");
        exit(1).unwrap();
    }
}
