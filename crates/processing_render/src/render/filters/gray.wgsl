#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct Settings {
    _padding: vec4<f32>,
}
@group(0) @binding(2) var<uniform> settings: Settings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = textureSample(screen_texture, texture_sampler, in.uv);
    // Processing 4 weights (PImage.java:990): 77/256, 151/256, 28/256
    let luma = 0.30078125 * c.r + 0.58984375 * c.g + 0.109375 * c.b;
    return vec4<f32>(luma, luma, luma, c.a);
}
