//! Spatial hash grid (`grid_create` / `grid_build`) against a CPU reference.

use std::collections::BTreeSet;

use processing::prelude::*;

const DIMS: [u32; 3] = [4, 4, 4];
const CELL_SIZE: f32 = 1.0;
const MIN: [f32; 3] = [0.0, 0.0, 0.0];

fn bytes_to_u32s(b: &[u8]) -> Vec<u32> {
    b.chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn cpu_cell(p: [f32; 3]) -> u32 {
    let rel = [
        (p[0] - MIN[0]) / CELL_SIZE,
        (p[1] - MIN[1]) / CELL_SIZE,
        (p[2] - MIN[2]) / CELL_SIZE,
    ];
    let clampi = |v: f32, m: u32| (v.floor() as i32).clamp(0, m as i32 - 1) as u32;
    let ix = clampi(rel[0], DIMS[0]);
    let iy = clampi(rel[1], DIMS[1]);
    let iz = clampi(rel[2], DIMS[2]);
    ix + iy * DIMS[0] + iz * DIMS[0] * DIMS[1]
}

fn sketch() -> error::Result<bool> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    let capacity: u32 = 40;
    let num_cells = (DIMS[0] * DIMS[1] * DIMS[2]) as usize;

    let cells: Vec<[u32; 3]> = (0..capacity)
        .map(|i| [(i * 7) % 4, (i * 13) % 4, (i * 5) % 4])
        .collect();
    let positions: Vec<[f32; 3]> = cells
        .iter()
        .map(|c| [c[0] as f32 + 0.5, c[1] as f32 + 0.5, c[2] as f32 + 0.5])
        .collect();

    let pos_bytes: Vec<u8> = positions
        .iter()
        .flat_map(|p| p.iter().flat_map(|f| f.to_le_bytes()))
        .collect();
    let position = buffer_create_with_data(pos_bytes)?;

    let params = GridParams {
        min: MIN,
        cell_size: CELL_SIZE,
        dims: DIMS,
    };
    let grid = grid_create(params, capacity)?;
    grid_build(grid, position)?;

    let offsets = bytes_to_u32s(&buffer_read(grid_get(grid)?.offsets)?);
    let sorted = bytes_to_u32s(&buffer_read(grid_get(grid)?.sorted)?);

    let mut counts = vec![0u32; num_cells];
    for i in 0..capacity as usize {
        counts[cpu_cell(positions[i]) as usize] += 1;
    }
    let mut expected_offsets = vec![0u32; num_cells + 1];
    for c in 0..num_cells {
        expected_offsets[c + 1] = expected_offsets[c] + counts[c];
    }

    let mut ok = true;

    if offsets != expected_offsets {
        ok = false;
        let i = (0..offsets.len())
            .find(|&i| offsets.get(i) != expected_offsets.get(i))
            .unwrap_or(0);
        println!(
            "  FAIL offsets: first mismatch at cell {i}: got {:?}, want {:?}",
            offsets.get(i),
            expected_offsets.get(i)
        );
    } else {
        println!("  PASS offsets (total={})", expected_offsets[num_cells]);
    }

    let mut buckets_ok = true;
    for c in 0..num_cells {
        let (s, e) = (
            expected_offsets[c] as usize,
            expected_offsets[c + 1] as usize,
        );
        let got: BTreeSet<u32> = sorted[s..e].iter().copied().collect();
        let want: BTreeSet<u32> = (0..capacity)
            .filter(|&i| cpu_cell(positions[i as usize]) as usize == c)
            .collect();
        if got != want {
            buckets_ok = false;
            println!("  FAIL cell {c}: got {got:?}, want {want:?}");
        }
    }
    if buckets_ok {
        println!("  PASS buckets ({num_cells} cells)");
    } else {
        ok = false;
    }

    grid_destroy(grid)?;
    if matches!(
        grid_build(grid, position),
        Err(error::ProcessingError::GridNotFound)
    ) {
        println!("  PASS destroyed grid is gone");
    } else {
        ok = false;
        println!("  FAIL destroyed grid still builds");
    }

    buffer_destroy(position)?;
    Ok(ok)
}

fn main() {
    let ok = sketch().unwrap();
    if ok {
        println!("grid_test: ALL PASS");
        exit(0).unwrap();
    } else {
        println!("grid_test: FAILURES");
        exit(1).unwrap();
    }
}
