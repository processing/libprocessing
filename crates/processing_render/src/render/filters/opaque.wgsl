import processing::filter::{sample, FullscreenVertexOutput};

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = sample(in.uv);
    return vec4<f32>(c.rgb, 1.0);
}
