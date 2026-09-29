import processing::particles::falloff;

struct Params {
    max_distance: f32,
    op: u32,
    falloff_mode: u32,
    components: u32,
    // 1 = reduce `source[j] - source[i]`
    relative: u32,
}

const OP_SUM: u32 = 0u;
const OP_MEAN: u32 = 1u;
const OP_COUNT: u32 = 2u;

@group(0) @binding(0) var<storage, read>       position:       array<f32>;
@group(0) @binding(1) var<storage, read>       source:         array<f32>;
@group(0) @binding(2) var<storage, read_write> out:            array<f32>;
@group(0) @binding(3) var<storage, read>       neighbors:      array<u32>;
@group(0) @binding(4) var<storage, read>       neighbor_count: array<u32>;
@group(0) @binding(5) var<uniform>             params:         Params;

fn load_pos(i: u32) -> vec3<f32> {
    return vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&neighbor_count);
    if i >= n { return; }

    let pos = load_pos(i);
    let r2 = params.max_distance * params.max_distance;
    let comps = params.components;

    var own = array<f32, 4>(0.0, 0.0, 0.0, 0.0);
    if params.relative == 1u {
        for (var c = 0u; c < comps; c++) {
            own[c] = source[i * comps + c];
        }
    }

    var value = array<f32, 4>(0.0, 0.0, 0.0, 0.0);
    var weight_sum = 0.0;
    let first = i * (arrayLength(&neighbors) / n);
    for (var k = 0u; k < neighbor_count[i]; k++) {
        let j = neighbors[first + k];
        let diff = load_pos(j) - pos;
        let d2 = dot(diff, diff);
        if d2 > r2 { continue; }
        let w = falloff(sqrt(d2), params.max_distance, params.falloff_mode);
        weight_sum += w;
        if params.op != OP_COUNT {
            for (var c = 0u; c < comps; c++) {
                value[c] += w * (source[j * comps + c] - own[c]);
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
