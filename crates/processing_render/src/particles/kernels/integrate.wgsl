struct Params {
    dt: f32,
}

@group(0) @binding(0) var<storage, read>       velocity: array<f32>;
@group(0) @binding(1) var<storage, read_write> position: array<f32>;
@group(0) @binding(2) var<uniform>             params:   Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let pi = i * 3u;
    position[pi]      = position[pi]      + velocity[pi]      * params.dt;
    position[pi + 1u] = position[pi + 1u] + velocity[pi + 1u] * params.dt;
    position[pi + 2u] = position[pi + 2u] + velocity[pi + 2u] * params.dt;
}
