import processing::filter::{sample_texture, FullscreenVertexOutput};

struct Params {
    dest_size: vec2<f32>,
    src_size: vec2<f32>,
    dest_rect: vec4<f32>,
    src_rect: vec4<f32>,
}
@group(1) @binding(0) var<uniform> params: Params;
@group(1) @binding(1) var dest_texture: texture_2d<f32>;
@group(1) @binding(2) var src_texture: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let st = in.uv;

    let dest = params.dest_rect.xy / params.dest_size + st * params.dest_rect.zw / params.dest_size;
    let src = params.src_rect.xy / params.src_size + st * params.src_rect.zw / params.src_size;

    let dest_color = sample_texture(dest_texture, dest).rgb;
    let src_color = sample_texture(src_texture, src).rgb;

    return vec4<f32>(dest_color / (1.0 - src_color), 1.0);
}
