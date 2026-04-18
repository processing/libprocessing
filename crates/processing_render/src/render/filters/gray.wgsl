import processing::filter::{sample, FullscreenVertexOutput};

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = sample(in.uv);
    let luma = 0.30078125 * c.r + 0.58984375 * c.g + 0.109375 * c.b;
    return vec4<f32>(luma, luma, luma, c.a);
}
