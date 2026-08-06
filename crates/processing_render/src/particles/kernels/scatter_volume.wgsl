struct Params {
    aabb_min: vec4<f32>,
    aabb_max: vec4<f32>,
    face_count: u32,
    max_attempts: u32,
    seed: u32,
    _pad: u32,
}

struct EmitRange {
    emit_base: u32,
    emit_count: u32,
    emit_capacity: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read>       source_position: array<f32>;
@group(0) @binding(1) var<storage, read>       source_indices:  array<u32>;
@group(0) @binding(2) var<storage, read_write> position:        array<f32>;
@group(0) @binding(3) var<storage, read_write> scale:           array<f32>;
@group(0) @binding(4) var<storage, read_write> age:             array<f32>;
@group(0) @binding(5) var<storage, read_write> life:            array<f32>;
@group(0) @binding(6) var<uniform>             params:          Params;
@group(0) @binding(7) var<uniform>             emit_range:      EmitRange;

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

fn fetch_vertex(i: u32) -> vec3<f32> {
    return vec3<f32>(
        source_position[i * 3u + 0u],
        source_position[i * 3u + 1u],
        source_position[i * 3u + 2u],
    );
}

fn ray_triangle(
    ro: vec3<f32>, rd: vec3<f32>,
    p0: vec3<f32>, p1: vec3<f32>, p2: vec3<f32>,
) -> f32 {
    let e1 = p1 - p0;
    let e2 = p2 - p0;
    let h  = cross(rd, e2);
    let a  = dot(e1, h);
    if abs(a) < 1e-7 { return -1.0; }
    let f = 1.0 / a;
    let s = ro - p0;
    let u = f * dot(s, h);
    if u < 0.0 || u > 1.0 { return -1.0; }
    let q = cross(s, e1);
    let v = f * dot(rd, q);
    if v < 0.0 || (u + v) > 1.0 { return -1.0; }
    return f * dot(e2, q);
}

fn point_inside(p: vec3<f32>) -> bool {
    let rd = normalize(vec3<f32>(0.5773, 0.5774, 0.5775));
    var hits = 0u;
    for (var f = 0u; f < params.face_count; f = f + 1u) {
        let i0 = source_indices[f * 3u + 0u];
        let i1 = source_indices[f * 3u + 1u];
        let i2 = source_indices[f * 3u + 2u];
        let t = ray_triangle(p, rd, fetch_vertex(i0), fetch_vertex(i1), fetch_vertex(i2));
        if t >= 0.0 {
            hits = hits + 1u;
        }
    }
    return (hits & 1u) == 1u;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let local_i = gid.x;
    if local_i >= emit_range.emit_count { return; }
    let base = emit_range.emit_base;
    let cap  = emit_range.emit_capacity;
    let slot = (base + local_i) % cap;
    let seed_base = params.seed ^ (base + local_i);

    let lo = params.aabb_min.xyz;
    let hi = params.aabb_max.xyz;

    var p = vec3<f32>(0.0);
    var found = false;
    for (var attempt = 0u; attempt < params.max_attempts; attempt = attempt + 1u) {
        let s = seed_base ^ (attempt * 0x9e3779b9u);
        p = vec3<f32>(
            mix(lo.x, hi.x, hash_unit(s * 7u  + 13u)),
            mix(lo.y, hi.y, hash_unit(s * 31u + 23u)),
            mix(lo.z, hi.z, hash_unit(s * 47u + 29u)),
        );
        if point_inside(p) {
            found = true;
            break;
        }
    }

    if !found { return; }

    position[slot * 3u + 0u] = p.x;
    position[slot * 3u + 1u] = p.y;
    position[slot * 3u + 2u] = p.z;
    scale[slot * 3u + 0u] = 1.0;
    scale[slot * 3u + 1u] = 1.0;
    scale[slot * 3u + 2u] = 1.0;
    age[slot] = 0.0;
    life[slot] = 1.0;
}
