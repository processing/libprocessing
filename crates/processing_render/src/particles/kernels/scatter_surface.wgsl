struct Params {
    face_count: u32,
    seed: u32,
    _pad0: u32,
    _pad1: u32,
}

struct EmitRange {
    emit_base: u32,
    emit_count: u32,
    emit_capacity: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read>       source_position: array<f32>;
@group(0) @binding(1) var<storage, read>       source_indices:  array<u32>;
@group(0) @binding(2) var<storage, read>       cdf:             array<f32>;
@group(0) @binding(3) var<storage, read_write> position:        array<f32>;
@group(0) @binding(4) var<storage, read_write> scale:           array<f32>;
@group(0) @binding(5) var<storage, read_write> age:             array<f32>;
@group(0) @binding(6) var<storage, read_write> life:            array<f32>;
@group(0) @binding(7) var<uniform>             params:          Params;
@group(0) @binding(8) var<uniform>             emit_range:      EmitRange;

fn hash(n: u32) -> u32 {
    var x = n;
    x = (x ^ 61u) ^ (x >> 16u);
    x = x + (x << 3u);
    x = x ^ (x >> 4u);
    x = x * 0x27d4eb2du;
    x = x ^ (x >> 15u);
    return x;
}

fn hash_unit(n: u32) -> f32 {
    return f32(hash(n)) / f32(0xffffffffu);
}

fn cdf_search(u: f32) -> u32 {
    var lo: u32 = 0u;
    var hi: u32 = params.face_count;
    loop {
        if lo >= hi { break; }
        let mid = (lo + hi) >> 1u;
        if cdf[mid] < u {
            lo = mid + 1u;
        } else {
            hi = mid;
        }
    }
    return min(lo, params.face_count - 1u);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let local_i = gid.x;
    if local_i >= emit_range.emit_count { return; }
    let base = emit_range.emit_base;
    let cap  = emit_range.emit_capacity;
    let slot = (base + local_i) % cap;
    let seed = params.seed ^ (base + local_i);

    let u01 = hash_unit(seed * 7u + 13u);
    let face = cdf_search(u01);

    let i0 = source_indices[face * 3u + 0u];
    let i1 = source_indices[face * 3u + 1u];
    let i2 = source_indices[face * 3u + 2u];

    let p0 = vec3<f32>(
        source_position[i0 * 3u + 0u],
        source_position[i0 * 3u + 1u],
        source_position[i0 * 3u + 2u],
    );
    let p1 = vec3<f32>(
        source_position[i1 * 3u + 0u],
        source_position[i1 * 3u + 1u],
        source_position[i1 * 3u + 2u],
    );
    let p2 = vec3<f32>(
        source_position[i2 * 3u + 0u],
        source_position[i2 * 3u + 1u],
        source_position[i2 * 3u + 2u],
    );

    var u = hash_unit(seed * 31u + 23u);
    var v = hash_unit(seed * 47u + 29u);
    if u + v > 1.0 { u = 1.0 - u; v = 1.0 - v; }
    let p = (1.0 - u - v) * p0 + u * p1 + v * p2;

    position[slot * 3u + 0u] = p.x;
    position[slot * 3u + 1u] = p.y;
    position[slot * 3u + 2u] = p.z;
    scale[slot * 3u + 0u] = 1.0;
    scale[slot * 3u + 1u] = 1.0;
    scale[slot * 3u + 2u] = 1.0;
    age[slot] = 0.0;
    life[slot] = 1.0;
}
