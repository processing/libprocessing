struct Params {
    nx: u32,
    ny: u32,
    time: f32,
    extent: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> color:    array<f32>;
@group(0) @binding(2) var<storage, read_write> normal:   array<f32>;
@group(0) @binding(3) var<storage, read_write> indices:  array<u32>;
@group(0) @binding(4) var<uniform>             params:   Params;

fn height(x: f32, z: f32, t: f32) -> f32 {
    let r = sqrt(x * x + z * z);
    return sin(r * 0.8 - t * 2.0) * 1.6 * exp(-r * 0.10)
         + sin(x * 0.5 + t) * 0.4
         + cos(z * 0.6 - t * 1.3) * 0.3;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let nx = params.nx;
    let ny = params.ny;
    if i >= nx * ny { return; }

    let gx = i % nx;
    let gy = i / nx;
    let x = (f32(gx) / f32(nx - 1u) - 0.5) * params.extent;
    let z = (f32(gy) / f32(ny - 1u) - 0.5) * params.extent;
    let t = params.time;

    let h = height(x, z, t);
    position[i * 3u + 0u] = x;
    position[i * 3u + 1u] = h;
    position[i * 3u + 2u] = z;

    let e = 0.12;
    let hl = height(x - e, z, t);
    let hr = height(x + e, z, t);
    let hd = height(x, z - e, t);
    let hu = height(x, z + e, t);
    let n = normalize(vec3<f32>(hl - hr, 2.0 * e, hd - hu));
    normal[i * 3u + 0u] = n.x;
    normal[i * 3u + 1u] = n.y;
    normal[i * 3u + 2u] = n.z;

    let c = clamp(h * 0.35 + 0.5, 0.0, 1.0);
    color[i * 4u + 0u] = 0.15 + 0.75 * c;
    color[i * 4u + 1u] = 0.30 + 0.45 * (1.0 - abs(c - 0.5) * 2.0);
    color[i * 4u + 2u] = 0.95 - 0.65 * c;
    color[i * 4u + 3u] = 1.0;

    if gx + 1u < nx && gy + 1u < ny {
        let cell = gy * (nx - 1u) + gx;
        let base = cell * 6u;
        indices[base + 0u] = i;
        indices[base + 1u] = i + 1u;
        indices[base + 2u] = i + nx + 1u;
        indices[base + 3u] = i;
        indices[base + 4u] = i + nx + 1u;
        indices[base + 5u] = i + nx;
    }
}
