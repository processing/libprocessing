@group(0) @binding(0) var<storage, read>       starts: array<u32>;
@group(0) @binding(1) var<storage, read_write> cursor: array<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= arrayLength(&cursor) { return; }
    cursor[i] = starts[i];
}
