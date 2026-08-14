// processing::prims — GPU-authored dynamic topology.
//
// A kernel imports this module and adds primitives to a `p.primitives(...)`
// target; the engine binds the target's buffers under the reserved names
// below (group 0, bindings 50-52) and draws the result indirectly, so the
// sketch never touches counts, draw arguments, or index buffers.
//
//     import { processing::prims };
//
//     var a = prims::vertex(pos_i);  a.color = vec4<f32>(rgb_i, alpha);
//     var b = prims::vertex(pos_j);  b.color = vec4<f32>(rgb_j, alpha);
//     prims::line(a, b);
//
// `line`/`tri`/`point` reserve vertex slots atomically, clamp against the
// target's capacity (over-capacity primitives are dropped, never partially
// written), and count every attempt so the sketch can poll
// `target.emitted()` / `target.overflowed()`.

// Indirect draw arguments with stats in the tail. The GPU's draw_indirect
// reads only the first four words; `instance_count` stays 1 and is written
// by the engine at frame reset.
struct PrimsArgs {
    vertex_count: atomic<u32>,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
    // Primitives kernels tried to add this frame (including dropped ones).
    attempted: atomic<u32>,
    // Capacity of the target, in vertices.
    capacity_verts: u32,
    // Vertices per primitive for the target's topology.
    verts_per_prim: u32,
    _pad: u32,
}

// Reserved bindings, filled by `apply(kernel, primitives=target)`.
// Bindings 50-52 in group 0 are reserved for this module; user kernels
// should number their own bindings below 50.
@group(0) @binding(50) var<storage, read_write> prims_out_position: array<f32>;
@group(0) @binding(51) var<storage, read_write> prims_out_color: array<f32>;
@group(0) @binding(52) var<storage, read_write> prims_args: PrimsArgs;

struct Vertex {
    position: vec3<f32>,
    color: vec4<f32>,
}

// A vertex at `p` with default white color; set `.color` before committing.
fn vertex(p: vec3<f32>) -> Vertex {
    return Vertex(p, vec4<f32>(1.0, 1.0, 1.0, 1.0));
}

const INVALID_SLOT: u32 = 0xffffffffu;

// Reserves `n` contiguous vertex slots. Returns the base slot, or
// INVALID_SLOT when the target is full (with the reservation rolled back so
// the drawn count never exceeds what was actually written).
fn reserve(n: u32) -> u32 {
    atomicAdd(&prims_args.attempted, 1u);
    let base = atomicAdd(&prims_args.vertex_count, n);
    if base + n > prims_args.capacity_verts {
        atomicSub(&prims_args.vertex_count, n);
        return INVALID_SLOT;
    }
    return base;
}

fn write_vertex(slot: u32, v: Vertex) {
    prims_out_position[slot * 3u + 0u] = v.position.x;
    prims_out_position[slot * 3u + 1u] = v.position.y;
    prims_out_position[slot * 3u + 2u] = v.position.z;
    prims_out_color[slot * 4u + 0u] = v.color.x;
    prims_out_color[slot * 4u + 1u] = v.color.y;
    prims_out_color[slot * 4u + 2u] = v.color.z;
    prims_out_color[slot * 4u + 3u] = v.color.w;
}

// Adds one point primitive to a POINTS target.
fn point(a: Vertex) {
    let base = reserve(1u);
    if base == INVALID_SLOT { return; }
    write_vertex(base, a);
}

// Adds one line segment to a LINES target.
fn line(a: Vertex, b: Vertex) {
    let base = reserve(2u);
    if base == INVALID_SLOT { return; }
    write_vertex(base, a);
    write_vertex(base + 1u, b);
}

// Adds one triangle to a TRIANGLES target.
fn tri(a: Vertex, b: Vertex, c: Vertex) {
    let base = reserve(3u);
    if base == INVALID_SLOT { return; }
    write_vertex(base, a);
    write_vertex(base + 1u, b);
    write_vertex(base + 2u, c);
}
