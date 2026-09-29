import processing::particles::{Grid, cell_coords, cell_index};

// with radius == cell size, ~1 in 6.5 candidates is a neighbor; rounded up
const CANDIDATES_PER_NEIGHBOR: f32 = 7.0;

struct Params {
    radius: f32,
    max_neighbors: u32,
}

@group(0) @binding(0) var<storage, read>       position:       array<f32>;
@group(0) @binding(1) var<storage, read>       grid_offsets:   array<u32>;
@group(0) @binding(2) var<storage, read>       grid_sorted:    array<u32>;
@group(0) @binding(3) var<uniform>             grid:           Grid;
@group(0) @binding(4) var<storage, read_write> neighbors:      array<u32>;
@group(0) @binding(5) var<storage, read_write> neighbor_count: array<u32>;
@group(0) @binding(6) var<uniform>             params:         Params;

fn load_pos(i: u32) -> vec3<f32> {
    return vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= arrayLength(&neighbor_count) { return; }

    let pos = load_pos(i);
    let r2 = params.radius * params.radius;
    let dims = grid.dims;
    let base = cell_coords(pos, grid.origin, grid.cell_size, dims);
    let reach = max(1, i32(ceil(params.radius / grid.cell_size)));
    let lo = max(base - vec3<i32>(reach), vec3<i32>(0));
    let hi = min(base + vec3<i32>(reach), vec3<i32>(dims) - vec3<i32>(1));

    // past the limit, sample every cell evenly so lists aren't biased toward the first cells
    var total = 0u;
    for (var cz = lo.z; cz <= hi.z; cz++) {
        for (var cy = lo.y; cy <= hi.y; cy++) {
            for (var cx = lo.x; cx <= hi.x; cx++) {
                let cell = cell_index(vec3<u32>(u32(cx), u32(cy), u32(cz)), dims);
                total += grid_offsets[cell + 1u] - grid_offsets[cell];
            }
        }
    }
    let rate = min(1.0, f32(params.max_neighbors) * CANDIDATES_PER_NEIGHBOR / f32(max(total, 1u)));
    // per-particle offset so particles in a cell don't all sample the same members
    let phase = i * 2654435761u;

    let first = i * params.max_neighbors;
    var found = 0u;
    for (var cz = lo.z; cz <= hi.z && found < params.max_neighbors; cz++) {
        for (var cy = lo.y; cy <= hi.y && found < params.max_neighbors; cy++) {
            for (var cx = lo.x; cx <= hi.x && found < params.max_neighbors; cx++) {
                let cell = cell_index(vec3<u32>(u32(cx), u32(cy), u32(cz)), dims);
                let start = grid_offsets[cell];
                let in_cell = grid_offsets[cell + 1u] - start;
                let take = min(in_cell, u32(ceil(f32(in_cell) * rate)));
                for (var k = 0u; k < take && found < params.max_neighbors; k++) {
                    var s = start + k;
                    if take < in_cell {
                        s = start + (phase + u32(f32(k) * f32(in_cell) / f32(take))) % in_cell;
                    }
                    let j = grid_sorted[s];
                    if j == i { continue; }
                    let diff = load_pos(j) - pos;
                    if dot(diff, diff) <= r2 {
                        neighbors[first + found] = j;
                        found += 1u;
                    }
                }
            }
        }
    }
    neighbor_count[i] = found;
}
