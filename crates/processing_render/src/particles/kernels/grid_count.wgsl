import processing::particles::cell_of;

struct Params {
    grid_min: vec3<f32>,
    cell_size: f32,
    dims_x: u32,
    dims_y: u32,
    dims_z: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read>            position: array<f32>;
@group(0) @binding(1) var<storage, read_write>      counts:   array<atomic<u32>>;
@group(0) @binding(2) var<uniform>                  params:   Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&position) / 3u;
    if i >= n { return; }
    let p = vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
    let dims = vec3<u32>(params.dims_x, params.dims_y, params.dims_z);
    atomicAdd(&counts[cell_of(p, params.grid_min, params.cell_size, dims)], 1u);
}
