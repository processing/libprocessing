@group(0) @binding(0) var<storage, read>       flags:   array<f32>;
@group(0) @binding(1) var<storage, read>       scanned: array<u32>;
@group(0) @binding(2) var<storage, read_write> indices: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= arrayLength(&flags) { return; }
    if flags[i] != 0.0 {
        indices[scanned[i]] = i;
    }
}
