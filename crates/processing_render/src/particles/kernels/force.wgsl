struct Params {
    direction: vec3<f32>,
    strength: f32,
}

@group(0) @binding(0) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&velocity) / 3u;
    if i >= count { return; }

    let len2 = dot(params.direction, params.direction);
    if len2 < 0.000001 || params.strength == 0.0 { return; }
    let kick = params.direction * (params.strength * inverseSqrt(len2));

    let pi = i * 3u;
    velocity[pi]      = velocity[pi]      + kick.x;
    velocity[pi + 1u] = velocity[pi + 1u] + kick.y;
    velocity[pi + 2u] = velocity[pi + 2u] + kick.z;
}
