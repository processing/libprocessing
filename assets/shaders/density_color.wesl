struct Params {
    scale: f32,
}

@group(0) @binding(0) var<storage, read>       density: array<f32>;
@group(0) @binding(1) var<storage, read_write> color:   array<f32>;
@group(0) @binding(2) var<uniform>             params:  Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= arrayLength(&density) { return; }

    let t = clamp(density[i] * params.scale, 0.0, 1.0);
    let cool = vec3<f32>(0.10, 0.22, 0.70);
    let warm = vec3<f32>(1.00, 0.55, 0.15);
    let rgb = mix(cool, warm, t) * (0.35 + 1.1 * t);

    color[i * 4u + 0u] = rgb.x;
    color[i * 4u + 1u] = rgb.y;
    color[i * 4u + 2u] = rgb.z;
    color[i * 4u + 3u] = 1.0;
}
