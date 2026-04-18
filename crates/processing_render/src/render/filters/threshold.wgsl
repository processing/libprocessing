import processing::filter::{sample, FullscreenVertexOutput};

struct Params {
    cutoff: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}
@group(1) @binding(0) var<uniform> params: Params;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = sample(in.uv);
    let m = max(c.r, max(c.g, c.b));
    let v = select(0.0, 1.0, m >= params.cutoff);
    return vec4<f32>(v, v, v, c.a);
}
