// Zeroes a primitives target's per-frame counts (drawn vertex count + the
// attempted stat) on the GPU, so the reset is ordered in the command stream
// ahead of the kernels that append — a CPU-side asset write would race the
// dispatches. Layout mirrors PrimsArgs in shaders/processing/prims.wesl;
// instance_count and the capacity tail are left untouched.

struct PrimsArgs {
    vertex_count: atomic<u32>,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
    attempted: atomic<u32>,
    capacity_verts: u32,
    verts_per_prim: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read_write> prims_args: PrimsArgs;

@compute @workgroup_size(1)
fn main() {
    atomicStore(&prims_args.vertex_count, 0u);
    atomicStore(&prims_args.attempted, 0u);
}
