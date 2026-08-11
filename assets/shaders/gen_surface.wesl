struct Params {
    nx: u32,
    ny: u32,
    time: f32,
    extent: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> indices: array<u32>;
@group(0) @binding(2) var<uniform> params: Params;

fn wave(p: vec2<f32>, dir: vec2<f32>, freq: f32, speed: f32, t: f32) -> f32 {
    return sin(dot(p, normalize(dir)) * freq + t * speed);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let nx = params.nx;
    let ny = params.ny;
    if i >= nx * ny { return; }

    let gx = i % nx;
    let gy = i / nx;

    let u = f32(gx) / f32(nx - 1u);
    let v = f32(gy) / f32(ny - 1u);
    let x = (u - 0.5) * params.extent;
    let z = (v - 0.5) * params.extent;

    let t = params.time;
    let p = vec2(x, z);

    let w1 = wave(p, vec2(1.0, 0.3),  0.6, 0.7, t) * 1.0;
    let w2 = wave(p, vec2(0.4, 1.0),  0.9, 0.9, t) * 0.6;
    let w3 = wave(p, vec2(-0.7, 0.6), 1.7, 1.4, t) * 0.3;
    let w4 = wave(p, vec2(0.9, -0.5), 2.3, 1.8, t) * 0.18;
    let w5 = wave(p, vec2(-0.2, -1.0), 4.1, 2.6, t) * 0.08;
    var h = w1 + w2 + w3 + w4 + w5;
    h = h + 0.25 * h * abs(h);

    position[i * 3u + 0u] = x;
    position[i * 3u + 1u] = h;
    position[i * 3u + 2u] = z;

    if gx + 1u < nx && gy + 1u < ny {
        let cell = gy * (nx - 1u) + gx;
        let base = cell * 6u;
        let v00 = i;
        let v10 = i + 1u;
        let v01 = i + nx;
        let v11 = i + nx + 1u;
        indices[base + 0u] = v00;
        indices[base + 1u] = v10;
        indices[base + 2u] = v11;
        indices[base + 3u] = v00;
        indices[base + 4u] = v11;
        indices[base + 5u] = v01;
    }
}