struct Params {
    t_scale: f32,
    t_offset: f32,
    t_clamp: u32,
}

@group(0) @binding(0) var<storage, read_write> op_a: array<f32>;
@group(0) @binding(1) var<storage, read> op_b: array<f32>;
@group(0) @binding(2) var<storage, read> op_t: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&op_a);
    if i >= count { return; }

    var t = op_t[i] * params.t_scale + params.t_offset;
    if params.t_clamp != 0u {
        t = clamp(t, 0.0, 1.0);
    }
    op_a[i] = mix(op_a[i], op_b[i], t);
}
