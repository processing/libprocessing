struct Params {
    center: vec3<f32>,
    _pad0: f32,
    strength: f32,
    radius: f32,
    falloff_mode: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let pi = i * 3u;
    let pos = vec3<f32>(position[pi], position[pi + 1u], position[pi + 2u]);
    let diff = params.center - pos;
    let d2 = dot(diff, diff);
    let r2 = params.radius * params.radius;
    if d2 > r2 || d2 < 0.000001 { return; }

    let d = sqrt(d2);
    let dir = diff / d;

    var fall: f32 = 1.0;
    let n = 1.0 - d / params.radius;
    if params.falloff_mode == 1u {
        fall = n;
    } else if params.falloff_mode == 2u {
        fall = n * n * (3.0 - 2.0 * n);
    } else if params.falloff_mode == 3u {
        fall = n * n;
    } else if params.falloff_mode == 4u {
        fall = n * n * n;
    } else if params.falloff_mode == 5u {
        fall = params.radius / (d + params.radius);
    }

    let kick = dir * (params.strength * fall);
    velocity[pi]      = velocity[pi]      + kick.x;
    velocity[pi + 1u] = velocity[pi + 1u] + kick.y;
    velocity[pi + 2u] = velocity[pi + 2u] + kick.z;
}
