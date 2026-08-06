struct Params {
    u_scale: f32,
    u_offset: f32,
    v_scale: f32,
    v_offset: f32,
    color_scale: f32,
}

@group(0) @binding(0) var<storage, read> op_in_u: array<f32>;
@group(0) @binding(1) var<storage, read> op_in_v: array<f32>;
@group(0) @binding(2) var<storage, read_write> op_out: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> params: Params;
@group(0) @binding(4) var lookup: texture_2d<f32>;
@group(0) @binding(5) var lookup_sampler: sampler;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&op_out);
    if i >= count { return; }

    let u = clamp(op_in_u[i] * params.u_scale + params.u_offset, 0.0, 1.0);
    let v = clamp(op_in_v[i] * params.v_scale + params.v_offset, 0.0, 1.0);
    let c = textureSampleLevel(lookup, lookup_sampler, vec2<f32>(u, v), 0.0);
    op_out[i] = vec4<f32>(c.rgb * params.color_scale, c.a);
}
