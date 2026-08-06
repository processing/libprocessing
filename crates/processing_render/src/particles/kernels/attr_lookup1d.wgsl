struct Params {
    scale: f32,
    offset: f32,
}

@group(0) @binding(0) var<storage, read> op_in: array<f32>;
@group(0) @binding(1) var<storage, read_write> op_out_r: array<f32>;
@group(0) @binding(2) var<storage, read_write> op_out_g: array<f32>;
@group(0) @binding(3) var<storage, read_write> op_out_b: array<f32>;
@group(0) @binding(4) var<storage, read_write> op_out_a: array<f32>;
@group(0) @binding(5) var<uniform> params: Params;
@group(0) @binding(6) var ramp: texture_2d<f32>;
@group(0) @binding(7) var ramp_sampler: sampler;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&op_in);
    if i >= count { return; }

    let t = clamp(op_in[i] * params.scale + params.offset, 0.0, 1.0);
    let c = textureSampleLevel(ramp, ramp_sampler, vec2<f32>(t, 0.5), 0.0);
    op_out_r[i] = c.r;
    op_out_g[i] = c.g;
    op_out_b[i] = c.b;
    op_out_a[i] = c.a;
}
