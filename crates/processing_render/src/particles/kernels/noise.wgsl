struct Params {
    scale: f32,
    strength: f32,
    time: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;

fn hash(p: vec3<f32>) -> f32 {
    let q = fract(p * 0.3183099) + vec3<f32>(0.1, 0.2, 0.3);
    let r = q + dot(q, q.yzx + 19.19);
    return fract(r.x * r.y * r.z);
}

fn value_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(
            mix(hash(i + vec3<f32>(0.0, 0.0, 0.0)),
                hash(i + vec3<f32>(1.0, 0.0, 0.0)), u.x),
            mix(hash(i + vec3<f32>(0.0, 1.0, 0.0)),
                hash(i + vec3<f32>(1.0, 1.0, 0.0)), u.x),
            u.y),
        mix(
            mix(hash(i + vec3<f32>(0.0, 0.0, 1.0)),
                hash(i + vec3<f32>(1.0, 0.0, 1.0)), u.x),
            mix(hash(i + vec3<f32>(0.0, 1.0, 1.0)),
                hash(i + vec3<f32>(1.0, 1.0, 1.0)), u.x),
            u.y),
        u.z);
}

fn noise3(p: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        value_noise(p),
        value_noise(p + vec3<f32>(31.4, 0.0, 0.0)),
        value_noise(p + vec3<f32>(0.0, 71.7, 0.0)),
    ) * 2.0 - 1.0;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count {
        return;
    }
    let p = vec3<f32>(
        position[i * 3u + 0u],
        position[i * 3u + 1u],
        position[i * 3u + 2u],
    );
    let sample = p * params.scale + vec3<f32>(params.time, params.time * 0.7, params.time * 1.3);
    let n = noise3(sample);
    let new_p = p + n * params.strength;
    position[i * 3u + 0u] = new_p.x;
    position[i * 3u + 1u] = new_p.y;
    position[i * 3u + 2u] = new_p.z;
}
