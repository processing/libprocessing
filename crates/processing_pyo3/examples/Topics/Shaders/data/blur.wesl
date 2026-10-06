import processing::filter::{sample, texel_size, FullscreenVertexOutput};

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let t = texel_size();
    let tc0 = in.uv + vec2<f32>(-t.x, -t.y);
    let tc1 = in.uv + vec2<f32>(0.0, -t.y);
    let tc2 = in.uv + vec2<f32>(t.x, -t.y);
    let tc3 = in.uv + vec2<f32>(-t.x, 0.0);
    let tc4 = in.uv;
    let tc5 = in.uv + vec2<f32>(t.x, 0.0);
    let tc6 = in.uv + vec2<f32>(-t.x, t.y);
    let tc7 = in.uv + vec2<f32>(0.0, t.y);
    let tc8 = in.uv + vec2<f32>(t.x, t.y);

    let col0 = sample(tc0);
    let col1 = sample(tc1);
    let col2 = sample(tc2);
    let col3 = sample(tc3);
    let col4 = sample(tc4);
    let col5 = sample(tc5);
    let col6 = sample(tc6);
    let col7 = sample(tc7);
    let col8 = sample(tc8);

    let sum = (1.0 * col0 + 2.0 * col1 + 1.0 * col2 +
               2.0 * col3 + 4.0 * col4 + 2.0 * col5 +
               1.0 * col6 + 2.0 * col7 + 1.0 * col8) / 16.0;
    return vec4<f32>(sum.rgb, 1.0);
}
