@group(0) @binding(0) var<storage, read>       flags:   array<f32>;
@group(0) @binding(1) var<storage, read_write> scanned: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let m = arrayLength(&scanned);
    if i >= m { return; }
    let n = arrayLength(&flags);
    if i < n {
        scanned[i] = select(0u, 1u, flags[i] != 0.0);
    } else {
        scanned[i] = 0u;
    }
}
