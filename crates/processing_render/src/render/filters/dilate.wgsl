import processing::filter::{sample, texel_size, FullscreenVertexOutput};

fn luma(c: vec3<f32>) -> f32 {
    return 0.30078125 * c.r + 0.58984375 * c.g + 0.109375 * c.b;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let t = texel_size();
    let offsets = array<vec2<f32>, 4>(
        vec2<f32>(-t.x, 0.0),
        vec2<f32>(t.x, 0.0),
        vec2<f32>(0.0, -t.y),
        vec2<f32>(0.0, t.y),
    );
    var best = sample(in.uv);
    var best_luma = luma(best.rgb);
    for (var i = 0; i < 4; i++) {
        let s = sample(in.uv + offsets[i]);
        let sl = luma(s.rgb);
        if sl > best_luma {
            best = s;
            best_luma = sl;
        }
    }
    return best;
}
