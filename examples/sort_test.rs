//! GPU bitonic sort (`bitonic_sort_by_key`): key order and payload permutation.

use processing::prelude::*;

fn u32s(b: &[u8]) -> Vec<u32> {
    b.chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
fn f32s(b: &[u8]) -> Vec<f32> {
    b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
fn f32_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}
fn u32_bytes(v: &[u32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn rnd(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(747796405).wrapping_add(2891336453);
    x = ((x >> ((x >> 28).wrapping_add(4))) ^ x).wrapping_mul(277803737);
    x = (x >> 22) ^ x;
    (x as f32) / (u32::MAX as f32)
}

fn run_case(n: u32) -> error::Result<bool> {
    let orig_keys: Vec<f32> = (0..n)
        .map(|i| rnd(i.wrapping_mul(2654435761).wrapping_add(7)))
        .collect();
    let idx: Vec<u32> = (0..n).collect();

    let keys = buffer_create_with_data(f32_bytes(&orig_keys))?;
    let payload = buffer_create_with_data(u32_bytes(&idx))?;
    bitonic_sort_by_key(keys, payload)?;
    let sorted_keys = f32s(&buffer_read(keys)?);
    let perm = u32s(&buffer_read(payload)?);
    buffer_destroy(keys)?;
    buffer_destroy(payload)?;

    if let Some(i) = (1..n as usize).find(|&i| sorted_keys[i] < sorted_keys[i - 1]) {
        println!(
            "  FAIL n={n}: not ascending at {i}: {} < {}",
            sorted_keys[i],
            sorted_keys[i - 1]
        );
        return Ok(false);
    }
    let mut seen = vec![false; n as usize];
    for &p in &perm {
        if p >= n || seen[p as usize] {
            println!("  FAIL n={n}: payload not a permutation (bad/dup index {p})");
            return Ok(false);
        }
        seen[p as usize] = true;
    }
    if let Some(i) = (0..n as usize).find(|&i| orig_keys[perm[i] as usize] != sorted_keys[i]) {
        println!(
            "  FAIL n={n}: payload[{i}] mismatch: orig[{}]={} != {}",
            perm[i], orig_keys[perm[i] as usize], sorted_keys[i]
        );
        return Ok(false);
    }
    println!("  PASS n={n}");
    Ok(true)
}

fn sketch() -> error::Result<bool> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    let mut ok = true;
    for &n in &[2u32, 8, 64, 1024, 4096, 65536] {
        ok &= run_case(n)?;
    }
    Ok(ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("sort_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("sort_test: FAILURES");
        exit(1).unwrap();
    }
}
