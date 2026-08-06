struct Params {
    center: vec3<f32>,
    radius: f32,
    position_kick: f32,
    velocity_kick: f32,
    falloff_mode: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let pi = i * 3u;
    let pos = vec3<f32>(position[pi], position[pi + 1u], position[pi + 2u]);
    let diff = pos - params.center;
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

    let pos_push = dir * (params.position_kick * fall);
    let vel_push = dir * (params.velocity_kick * fall);

    position[pi]      = position[pi]      + pos_push.x;
    position[pi + 1u] = position[pi + 1u] + pos_push.y;
    position[pi + 2u] = position[pi + 2u] + pos_push.z;
    velocity[pi]      = velocity[pi]      + vel_push.x;
    velocity[pi + 1u] = velocity[pi + 1u] + vel_push.y;
    velocity[pi + 2u] = velocity[pi + 2u] + vel_push.z;
}
