import processing::filter::{sample, sample_texture, FullscreenVertexOutput};

@group(1) @binding(0) var mask: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let tex_color = sample(in.uv);
    let mask_color = sample_texture(mask, in.uv);
    return mix(tex_color, vec4<f32>(0.0, 0.0, 0.0, 0.0), 1.0 - mask_color.r);
}
