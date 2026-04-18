import processing::filter::{sample, FullscreenVertexOutput};

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = sample(in.uv);
    return vec4<f32>(1.0 - c.r, 1.0 - c.g, 1.0 - c.b, c.a);
}
