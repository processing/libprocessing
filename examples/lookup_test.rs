//! `lookup`: per-particle texture sampling, 1-D and 2-D.

use bevy::prelude::Entity;
use bevy::render::render_resource::Extent3d;
use processing::prelude::*;

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];

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

fn ramp_image(width: u32, height: u32, texels: &[[u8; 4]]) -> error::Result<Entity> {
    let data: Vec<u8> = texels.iter().flatten().copied().collect();
    let img = image_create(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        data,
        TextureFormat::Rgba8Unorm,
    )?;
    image_set_sampler(img, 1, 0, 0)?; // nearest, clamp
    Ok(img)
}

fn sketch() -> error::Result<bool> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    let mut ok = true;

    let ramp = ramp_image(4, 1, &[RED, GREEN, BLUE, WHITE])?;
    let t = vec![0.125f32, 0.375, 0.625, 0.875];
    let op_in = buffer_create_with_data(to_bytes(&t))?;
    let dst = buffer_create_with_data(to_bytes(&vec![0.0; 16]))?;
    lookup(dst, op_in, ramp, 1, 1.0, 0.0, 1.0, 0.0, 1.0)?;
    let got = f32s(&buffer_read(dst)?);
    let want = vec![
        1.0, 0.0, 0.0, 1.0, // red
        0.0, 1.0, 0.0, 1.0, // green
        0.0, 0.0, 1.0, 1.0, // blue
        1.0, 1.0, 1.0, 1.0, // white
    ];
    if approx(&got, &want) {
        println!("  PASS lookup 1-D ramp");
    } else {
        ok = false;
        println!("  FAIL lookup 1-D: got {got:?}");
    }
    buffer_destroy(op_in)?;
    buffer_destroy(dst)?;

    let tex2 = ramp_image(2, 2, &[RED, GREEN, BLUE, WHITE])?;
    let uv = vec![
        0.25f32, 0.25, // red
        0.75, 0.25, // green
        0.25, 0.75, // blue
        0.75, 0.75, // white
    ];
    let op_in2 = buffer_create_with_data(to_bytes(&uv))?;
    let dst2 = buffer_create_with_data(to_bytes(&vec![0.0; 16]))?;
    lookup(dst2, op_in2, tex2, 2, 1.0, 0.0, 1.0, 0.0, 1.0)?;
    let got2 = f32s(&buffer_read(dst2)?);
    if approx(&got2, &want) {
        println!("  PASS lookup 2-D texture");
    } else {
        ok = false;
        println!("  FAIL lookup 2-D: got {got2:?}");
    }
    buffer_destroy(op_in2)?;
    buffer_destroy(dst2)?;

    Ok(ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("lookup_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("lookup_test: FAILURES");
        exit(1).unwrap();
    }
}
