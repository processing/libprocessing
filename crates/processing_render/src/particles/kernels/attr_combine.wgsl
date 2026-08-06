struct Params {
    op: u32,
    b_scale: f32,
    b_offset: f32,
}

@group(0) @binding(0) var<storage, read_write> op_a: array<f32>;
@group(0) @binding(1) var<storage, read> op_b: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&op_a);
    if i >= count { return; }

    let a = op_a[i];
    let b = op_b[i] * params.b_scale + params.b_offset;

    var r: f32;
    switch params.op {
        case 0u: { r = a + b; }
        case 1u: { r = a - b; }
        case 2u: { r = a * b; }
        case 3u: {
            if b == 0.0 { r = a; } else { r = a / b; }
        }
        case 4u: { r = min(a, b); }
        case 5u: { r = max(a, b); }
        case 6u: { r = pow(max(a, 0.0), b); }
        default: { r = a; }
    }
    op_a[i] = r;
}
