import processing::particles::{Grid, cell_coords, cell_index, falloff};

struct Params {
    radius: f32,
    op: u32,
    falloff_mode: u32,
    components: u32,
}

const OP_SUM: u32 = 0u;
const OP_MEAN: u32 = 1u;
const OP_COUNT: u32 = 2u;

@group(0) @binding(0) var<storage, read>       position:     array<f32>;
@group(0) @binding(1) var<storage, read>       source:       array<f32>;
@group(0) @binding(2) var<storage, read_write> out:          array<f32>;
@group(0) @binding(3) var<storage, read>       grid_offsets: array<u32>;
@group(0) @binding(4) var<storage, read>       grid_sorted:  array<u32>;
@group(0) @binding(5) var<uniform>             params:       Params;
@group(0) @binding(6) var<uniform>             grid:         Grid;

fn load_pos(i: u32) -> vec3<f32> {
    return vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let pos = load_pos(i);
    let r2 = params.radius * params.radius;
    let comps = params.components;

    let dims = grid.dims;
    let base = cell_coords(pos, grid.origin, grid.cell_size, dims);

    var value = array<f32, 4>(0.0, 0.0, 0.0, 0.0);
    var weight_sum = 0.0;

    for (var dz = -1; dz <= 1; dz++) {
        let cz = base.z + dz;
        if cz < 0 || cz >= i32(grid.dims.z) { continue; }
        for (var dy = -1; dy <= 1; dy++) {
            let cy = base.y + dy;
            if cy < 0 || cy >= i32(grid.dims.y) { continue; }
            for (var dx = -1; dx <= 1; dx++) {
                let cx = base.x + dx;
                if cx < 0 || cx >= i32(grid.dims.x) { continue; }

                let cell = cell_index(vec3<u32>(u32(cx), u32(cy), u32(cz)), dims);
                let start = grid_offsets[cell];
                let end = grid_offsets[cell + 1u];
                for (var s = start; s < end; s++) {
                    let j = grid_sorted[s];
                    let diff = pos - load_pos(j);
                    let d2 = dot(diff, diff);
                    if d2 <= r2 {
                        let w = falloff(sqrt(d2), params.radius, params.falloff_mode);
                        weight_sum += w;
                        if params.op != OP_COUNT {
                            for (var c = 0u; c < comps; c++) {
                                value[c] += w * source[j * comps + c];
                            }
                        }
                    }
                }
            }
        }
    }

    if params.op == OP_COUNT {
        out[i] = weight_sum;
    } else if params.op == OP_MEAN {
        let inv = select(0.0, 1.0 / weight_sum, weight_sum > 0.0);
        for (var c = 0u; c < comps; c++) {
            out[i * comps + c] = value[c] * inv;
        }
    } else {
        for (var c = 0u; c < comps; c++) {
            out[i * comps + c] = value[c];
        }
    }
}
