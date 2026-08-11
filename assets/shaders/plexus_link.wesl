struct GridParams {
    grid_min: vec3<f32>,
    cell_size: f32,
    dims_x: u32,
    dims_y: u32,
    dims_z: u32,
    _pad: u32,
}

struct Params {
    connection_radius: f32,
    connection_ramp: f32,  // falloff exponent
    line_alpha: f32,       // opacity scale
    max_links: u32,        // per-particle edge cap
}

// Auto-bound by apply(): names must be position/color.
@group(0) @binding(0) var<storage, read>       position:  array<f32>;
@group(0) @binding(1) var<storage, read>       color:     array<f32>;
@group(0) @binding(2) var<storage, read_write> edge_pos:  array<f32>;
@group(0) @binding(3) var<storage, read_write> edge_col:  array<f32>;
@group(0) @binding(4) var<storage, read_write> indices:   array<u32>;
@group(0) @binding(5) var<storage, read_write> draw_args: array<atomic<u32>>;
// Bound by grid.bind().
@group(0) @binding(6) var<storage, read>       offsets:   array<u32>;
@group(0) @binding(7) var<storage, read>       sorted:    array<u32>;
@group(0) @binding(8) var<uniform>             params:    Params;
@group(0) @binding(9) var<uniform>             gp:        GridParams;

fn cell_coords(p: vec3<f32>, grid_min: vec3<f32>, cell_size: f32, dims: vec3<u32>) -> vec3<i32> {
    let rel = (p - grid_min) / cell_size;
    return vec3<i32>(
        clamp(i32(floor(rel.x)), 0, i32(dims.x) - 1),
        clamp(i32(floor(rel.y)), 0, i32(dims.y) - 1),
        clamp(i32(floor(rel.z)), 0, i32(dims.z) - 1),
    );
}

fn cell_index(c: vec3<u32>, dims: vec3<u32>) -> u32 {
    return c.x + c.y * dims.x + c.z * dims.x * dims.y;
}

fn load_pos(i: u32) -> vec3<f32> {
    return vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
}

fn load_rgb(i: u32) -> vec3<f32> {
    return vec3<f32>(color[i * 4u], color[i * 4u + 1u], color[i * 4u + 2u]);
}

fn emit_vertex(slot: u32, p: vec3<f32>, rgb: vec3<f32>, a: f32) {
    edge_pos[slot * 3u + 0u] = p.x;
    edge_pos[slot * 3u + 1u] = p.y;
    edge_pos[slot * 3u + 2u] = p.z;
    edge_col[slot * 4u + 0u] = rgb.x;
    edge_col[slot * 4u + 1u] = rgb.y;
    edge_col[slot * 4u + 2u] = rgb.z;
    edge_col[slot * 4u + 3u] = a;
    indices[slot] = slot;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let cap = arrayLength(&indices);
    let pos = load_pos(i);
    let rgb = load_rgb(i);
    let radius = params.connection_radius;
    let r2 = radius * radius;
    let dims = vec3<u32>(gp.dims_x, gp.dims_y, gp.dims_z);
    let base = cell_coords(pos, gp.grid_min, gp.cell_size, dims);

    let reach = max(1, i32(ceil(radius / gp.cell_size)));
    let z0 = max(base.z - reach, 0);
    let z1 = min(base.z + reach, i32(gp.dims_z) - 1);
    let y0 = max(base.y - reach, 0);
    let y1 = min(base.y + reach, i32(gp.dims_y) - 1);
    let x0 = max(base.x - reach, 0);
    let x1 = min(base.x + reach, i32(gp.dims_x) - 1);

    var emitted = 0u;
    for (var cz = z0; cz <= z1; cz++) {
        for (var cy = y0; cy <= y1; cy++) {
            for (var cx = x0; cx <= x1; cx++) {
                let cell = cell_index(vec3<u32>(u32(cx), u32(cy), u32(cz)), dims);
                let start = offsets[cell];
                let end = offsets[cell + 1u];
                for (var s = start; s < end; s++) {
                    let j = sorted[s];
                    if j <= i { continue; }              // one direction per edge
                    let pj = load_pos(j);
                    let diff = pos - pj;
                    let d2 = dot(diff, diff);
                    if d2 > r2 { continue; }

                    if emitted >= params.max_links { return; }
                    emitted += 1u;

                    let d = sqrt(d2);
                    let a = pow(1.0 / (d / radius + 1.0), params.connection_ramp) * params.line_alpha;

                    let slot = atomicAdd(&draw_args[0], 2u);
                    if slot + 1u < cap {
                        emit_vertex(slot, pos, rgb, a);
                        emit_vertex(slot + 1u, pj, load_rgb(j), a);
                    }
                }
            }
        }
    }
}
