import processing::filter::{sample, FullscreenVertexOutput};

struct Params {
    levels: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}
@group(1) @binding(0) var<uniform> params: Params;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = sample(in.uv);
    let n = f32(max(params.levels, 2u));
    let q = floor(c.rgb * n) / (n - 1.0);
    return vec4<f32>(clamp(q, vec3<f32>(0.0), vec3<f32>(1.0)), c.a);
}
