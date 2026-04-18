struct ProcessingFilter {
    resolution: vec2<f32>,
    // 1.0 / resolution — one-texel step for neighbor sampling.
    texel_size: vec2<f32>,
    // Pass index (0-based) and total count.
    pass_index: u32,
    pass_count: u32,
    _pad: vec2<u32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> filter: ProcessingFilter;

struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn sample(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(screen_texture, texture_sampler, uv);
}

fn resolution() -> vec2<f32> { return filter.resolution; }
fn texel_size() -> vec2<f32> { return filter.texel_size; }
fn pass_index() -> u32 { return filter.pass_index; }
fn pass_count() -> u32 { return filter.pass_count; }
