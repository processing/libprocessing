struct Params {
    dt: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count {
        return;
    }
    let cs = cos(params.dt);
    let sn = sin(params.dt);
    let x = position[i * 3u + 0u];
    let z = position[i * 3u + 2u];
    position[i * 3u + 0u] = x * cs - z * sn;
    position[i * 3u + 2u] = x * sn + z * cs;
}
