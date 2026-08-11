import { lygia::color::space::hsv2rgb::hsv2rgb };

struct Params {
    count: u32,
    time: f32,
    fx: f32,        // frequency ratios
    fy: f32,
    fz: f32,
    loops: f32,     // parameter spans loops * TAU
    scale: f32,
    hue_mix: f32,   // 0 = grayscale (black), 1 = rainbow
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> color:    array<f32>;
@group(0) @binding(2) var<uniform>             params:   Params;

const TAU: f32 = 6.28318530718;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= params.count { return; }

    let u = f32(i) / f32(params.count);
    let t = u * TAU * params.loops;
    let world = vec3<f32>(
        sin(params.fx * t + params.time * 0.30),
        sin(params.fy * t + params.time * 0.47),
        sin(params.fz * t + params.time * 0.23),
    ) * params.scale;
    position[i * 3u + 0u] = world.x;
    position[i * 3u + 1u] = world.y;
    position[i * 3u + 2u] = world.z;

    let rainbow = hsv2rgb(vec3<f32>(fract(u + params.time * 0.03), 0.75, 1.0));
    let rgb = mix(vec3<f32>(0.0), rainbow, params.hue_mix);
    color[i * 4u + 0u] = rgb.x;
    color[i * 4u + 1u] = rgb.y;
    color[i * 4u + 2u] = rgb.z;
    color[i * 4u + 3u] = 1.0;
}
