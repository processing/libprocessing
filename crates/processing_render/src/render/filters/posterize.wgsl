#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct Settings {
    levels: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}
@group(0) @binding(2) var<uniform> settings: Settings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = textureSample(screen_texture, texture_sampler, in.uv);
    let n = f32(max(settings.levels, 2u));
    let q = floor(c.rgb * n) / (n - 1.0);
    return vec4<f32>(clamp(q, vec3<f32>(0.0), vec3<f32>(1.0)), c.a);
}
