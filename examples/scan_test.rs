//! GPU exclusive prefix-sum (`prefix_sum_u32`) against a CPU scan.

use processing::prelude::*;

fn u32s_to_bytes(v: &[u32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn bytes_to_u32s(b: &[u8]) -> Vec<u32> {
    b.chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn run_case(n: usize) -> error::Result<bool> {
    let input: Vec<u32> = (0..n).map(|i| (i % 7 + 1) as u32).collect();

    let buf = buffer_create_with_data(u32s_to_bytes(&input))?;
    prefix_sum_u32(buf)?;
    let out = bytes_to_u32s(&buffer_read(buf)?);
    buffer_destroy(buf)?;

    let mut expected = vec![0u32; n];
    let mut acc = 0u32;
    for i in 0..n {
        expected[i] = acc;
        acc += input[i];
    }

    if out.len() != expected.len() {
        println!("  FAIL n={n}: length {} != {}", out.len(), expected.len());
        return Ok(false);
    }
    if let Some(i) = (0..n).find(|&i| out[i] != expected[i]) {
        println!(
            "  FAIL n={n}: first mismatch at index {i}: got {}, want {}",
            out[i], expected[i]
        );
        return Ok(false);
    }
    println!("  PASS n={n} (total={acc})");
    Ok(true)
}

fn sketch() -> error::Result<bool> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    // empty, block boundary, two levels
    let cases = [1usize, 5, 255, 256, 257, 1000, 65_536, 70_000, 300_000];
    let mut all_ok = true;
    for &n in &cases {
        all_ok &= run_case(n)?;
    }
    Ok(all_ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("scan_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("scan_test: FAILURES");
        exit(1).unwrap();
    }
}
