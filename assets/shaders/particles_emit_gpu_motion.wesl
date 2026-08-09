struct Params {
    dt: f32,
    ttl: f32,
    gravity: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> scale: array<f32>;
@group(0) @binding(3) var<storage, read_write> age: array<f32>;
@group(0) @binding(4) var<storage, read_write> life: array<f32>;
@group(0) @binding(5) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count { return; }
    if life[i] <= 0.0 { return; }

    age[i] = age[i] + params.dt;

    velocity[i * 3u + 1u] = velocity[i * 3u + 1u] - params.gravity * params.dt;

    position[i * 3u + 0u] = position[i * 3u + 0u] + velocity[i * 3u + 0u] * params.dt;
    position[i * 3u + 1u] = position[i * 3u + 1u] + velocity[i * 3u + 1u] * params.dt;
    position[i * 3u + 2u] = position[i * 3u + 2u] + velocity[i * 3u + 2u] * params.dt;

    let remaining = clamp(1.0 - age[i] / params.ttl, 0.0, 1.0);
    let s = remaining * remaining;
    scale[i * 3u + 0u] = s;
    scale[i * 3u + 1u] = s;
    scale[i * 3u + 2u] = s;

    if age[i] > params.ttl { life[i] = 0.0; }
}
