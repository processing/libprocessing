// Conway's game of life
import processing::filter::{sample, texel_size, FullscreenVertexOutput};

struct Params {
    time: f32,
    _p0: f32,
    mouse: vec2<f32>,
}
@group(1) @binding(0) var<uniform> params: Params;

const live = vec4<f32>(0.5, 1.0, 0.7, 1.0);
const dead = vec4<f32>(0.0, 0.0, 0.0, 1.0);
const blue = vec4<f32>(0.0, 0.0, 1.0, 1.0);

// GLSL mod(): floored modulo (WGSL `%` truncates)
fn mod_f(x: f32, y: f32) -> f32 { return x - y * floor(x / y); }

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let position = in.uv;
    let pixel = texel_size();

    if (length(position - params.mouse) < 0.01) {
        let rnd1 = mod_f(fract(sin(dot(position + params.time * 0.001, vec2<f32>(14.9898, 78.233))) * 43758.5453), 1.0);
        if (rnd1 > 0.5) {
            return live;
        } else {
            return blue;
        }
    }

    // sample() reads the canvas as it was before this filter: the previous frame (ppixels)
    var sum = 0.0;
    sum += sample(position + pixel * vec2<f32>(-1.0, -1.0)).g;
    sum += sample(position + pixel * vec2<f32>(-1.0, 0.0)).g;
    sum += sample(position + pixel * vec2<f32>(-1.0, 1.0)).g;
    sum += sample(position + pixel * vec2<f32>(1.0, -1.0)).g;
    sum += sample(position + pixel * vec2<f32>(1.0, 0.0)).g;
    sum += sample(position + pixel * vec2<f32>(1.0, 1.0)).g;
    sum += sample(position + pixel * vec2<f32>(0.0, -1.0)).g;
    sum += sample(position + pixel * vec2<f32>(0.0, 1.0)).g;
    let me = sample(position);

    if (me.g <= 0.1) {
        if ((sum >= 2.9) && (sum <= 3.1)) {
            return live;
        } else if (me.b > 0.004) {
            return vec4<f32>(0.0, 0.0, max(me.b - 0.004, 0.25), 0.0);
        } else {
            return dead;
        }
    } else {
        if ((sum >= 1.9) && (sum <= 3.1)) {
            return live;
        } else {
            return blue;
        }
    }
}
