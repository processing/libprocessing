import processing::filter::{sample_texture, resolution, FullscreenVertexOutput};

struct Params {
    time: f32,
    _p0: f32,
    mouse: vec2<f32>,
}
@group(1) @binding(0) var<uniform> params: Params;
@group(1) @binding(1) var tex: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // gl_FragCoord has y pointing up; the filter uv has y pointing down
    let frag_coord = vec2<f32>(in.uv.x, 1.0 - in.uv.y) * resolution();
    let p = -1.0 + 2.0 * frag_coord / resolution();
    let m = -1.0 + 2.0 * params.mouse / resolution();

    let a1 = atan2(p.y - m.y, p.x - m.x);
    let r1 = sqrt(dot(p - m, p - m));
    let a2 = atan2(p.y + m.y, p.x + m.x);
    let r2 = sqrt(dot(p + m, p + m));

    var uv: vec2<f32>;
    uv.x = 0.2 * params.time + (r1 - r2) * 0.25;
    uv.y = sin(2.0 * (a1 - a2));

    let w = r1 * r2 * 0.8;
    // fract() wraps the texture like textureWrap(REPEAT)
    let col = sample_texture(tex, fract(0.5 - 0.495 * uv)).xyz;

    return vec4<f32>(col / (0.1 + w), 1.0);
}
