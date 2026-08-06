struct Params {
    center: vec3<f32>,
    radius: f32,
    falloff_mode: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<storage, read> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> weight: array<f32>;
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

    var w: f32 = 0.0;
    if d2 < r2 {
        let d = sqrt(d2);
        let n = 1.0 - d / params.radius;
        w = 1.0;
        if params.falloff_mode == 1u {
            w = n;
        } else if params.falloff_mode == 2u {
            w = n * n * (3.0 - 2.0 * n);
        } else if params.falloff_mode == 3u {
            w = n * n;
        } else if params.falloff_mode == 4u {
            w = n * n * n;
        } else if params.falloff_mode == 5u {
            w = params.radius / (d + params.radius);
        }
    }

    weight[i] = w;
}
