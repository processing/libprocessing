import processing::filter::{sample, texel_size, resolution, pass_index, FullscreenVertexOutput};

struct Params {
    ink: vec3<f32>,    // colour of the contour line
    spread: f32,       // how far each gold point is smeared, as a fraction of the height
    level: f32,        // the line runs where the smeared gold crosses this amount
    weight: f32,       // line thickness in pixels
}
@group(1) @binding(0) var<uniform> params: Params;

// how gold a pixel is: 0 for background and blue dust, 1 for the packed ridges
fn gold(c: vec3<f32>) -> f32 {
    let balance = (c.r - c.b) / max(c.r + c.b, 0.0001);
    return smoothstep(0.1, 0.4, balance);
}

// the first two passes smear the gold sideways and then upwards, keeping the result in alpha
fn smear(uv: vec2<f32>, dir: vec2<f32>, first: bool) -> vec4<f32> {
    let sigma = params.spread * resolution().y;
    let reach = i32(ceil(sigma * 2.5));
    var sum = 0.0;
    var weight_sum = 0.0;
    for (var i = -reach; i <= reach; i++) {
        let x = f32(i);
        let w = exp(-(x * x) / (2.0 * sigma * sigma));
        let c = sample(uv + dir * x);
        if first {
            sum += gold(c.rgb) * w;
        } else {
            sum += c.a * w;
        }
        weight_sum += w;
    }
    return vec4<f32>(sample(uv).rgb, sum / weight_sum);
}

// the last pass draws a line wherever the smeared gold crosses the level
fn outline(uv: vec2<f32>) -> vec4<f32> {
    let t = texel_size();
    let here = sample(uv);
    let dx = sample(uv + vec2<f32>(t.x, 0.0)).a - sample(uv - vec2<f32>(t.x, 0.0)).a;
    let dy = sample(uv + vec2<f32>(0.0, t.y)).a - sample(uv - vec2<f32>(0.0, t.y)).a;
    let slope = max(length(vec2<f32>(dx, dy)) * 0.5, 0.00001);

    // distance to the crossing, measured in pixels
    let away = abs(here.a - params.level) / slope;
    let line = 1.0 - smoothstep(params.weight * 0.5, params.weight * 0.5 + 1.0, away);
    return vec4<f32>(mix(here.rgb, params.ink, line), 1.0);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let t = texel_size();
    if pass_index() == 0u {
        return smear(in.uv, vec2<f32>(t.x, 0.0), true);
    }
    if pass_index() == 1u {
        return smear(in.uv, vec2<f32>(0.0, t.y), false);
    }
    return outline(in.uv);
}
