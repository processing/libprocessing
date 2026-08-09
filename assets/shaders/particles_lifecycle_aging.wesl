@group(0) @binding(0) var<storage, read_write> age: array<f32>;
@group(0) @binding(1) var<storage, read_write> life: array<f32>;
@group(0) @binding(2) var<storage, read_write> position: array<f32>;
@group(0) @binding(3) var<storage, read_write> scale: array<f32>;
@group(0) @binding(4) var<uniform> params: vec4<f32>;  // x = dt, y = ttl

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count {
        return;
    }
    let dt = params.x;
    let ttl = params.y;

    if life[i] <= 0.0 {
        return;
    }

    age[i] = age[i] + dt;
    position[i * 3u + 1u] = position[i * 3u + 1u] - dt * 1.5;

    let remaining = clamp(1.0 - age[i] / ttl, 0.0, 1.0);
    let s = remaining * remaining;
    scale[i * 3u + 0u] = s;
    scale[i * 3u + 1u] = s;
    scale[i * 3u + 2u] = s;

    if age[i] > ttl {
        life[i] = 0.0;
    }
}
