//! Stream compaction (`compact`) and the group predicate chain.

use bevy::prelude::Entity;
use processing::prelude::*;

fn u32s(b: &[u8]) -> Vec<u32> {
    b.chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
fn f32_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn check(label: &str, got_count: u32, got: &[u32], want: &[u32]) -> bool {
    if got_count as usize == want.len() && &got[..want.len()] == want {
        println!(
            "  PASS {label}: count={got_count}, indices={:?}",
            &got[..want.len()]
        );
        true
    } else {
        println!("  FAIL {label}: count={got_count} indices={got:?}, want {want:?}");
        false
    }
}

fn compact_flags(flags: &[f32]) -> error::Result<(u32, Vec<u32>)> {
    let f = buffer_create_with_data(f32_bytes(flags))?;
    let idx = buffer_create((flags.len().max(1) as u64) * 4)?;
    let count = compact(f, idx)?;
    let out = u32s(&buffer_read(idx)?);
    buffer_destroy(f)?;
    buffer_destroy(idx)?;
    Ok((count, out))
}

fn sketch() -> error::Result<bool> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    let mut ok = true;

    let (c, out) = compact_flags(&[0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0])?;
    ok &= check("compact mixed", c, &out, &[1, 2, 4, 7]);

    let (c, out) = compact_flags(&[0.0, 0.0, 0.0])?;
    ok &= check("compact none", c, &out, &[]);

    let (c, out) = compact_flags(&[1.0, 1.0, 1.0, 1.0])?;
    ok &= check("compact all", c, &out, &[0, 1, 2, 3]);

    let src_vals = [0.1f32, 0.5, 0.9, 0.3, 0.7];
    let src = buffer_create_with_data(f32_bytes(&src_vals))?;
    let flags = buffer_create((src_vals.len() as u64) * 4)?;
    let idx = buffer_create((src_vals.len() as u64) * 4)?;
    map(flags, src, 1, MAP_GREATER, 0.4, 0.0)?;
    let count = compact(flags, idx)?;
    let out = u32s(&buffer_read(idx)?);
    ok &= check("group(>0.4)+compact", count, &out, &[1, 2, 4]);
    for b in [src, flags, idx] {
        let _: Entity = b;
        buffer_destroy(b)?;
    }

    Ok(ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("group_compact_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("group_compact_test: FAILURES");
        exit(1).unwrap();
    }
}
