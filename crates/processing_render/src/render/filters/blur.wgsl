import processing::filter::{sample, texel_size, pass_index, FullscreenVertexOutput};

struct Params {
    radius: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}
@group(1) @binding(0) var<uniform> params: Params;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let t = texel_size();
    let dir = select(vec2<f32>(0.0, t.y), vec2<f32>(t.x, 0.0), pass_index() == 0u);

    let radius = max(params.radius, 1.0);
    let sigma = radius * 0.5 + 0.5;
    let r = i32(radius);

    var sum = vec4<f32>(0.0);
    var weight_sum = 0.0;
    for (var i = -r; i <= r; i++) {
        let x = f32(i);
        let w = exp(-(x * x) / (2.0 * sigma * sigma));
        sum += sample(in.uv + dir * x) * w;
        weight_sum += w;
    }
    return sum / weight_sum;
}
