// Adapted from:
// http://callumhay.blogspot.com/2010/09/gaussian-blur-shader-glsl.html
import processing::filter::{sample, texel_size, FullscreenVertexOutput};

struct Params {
    blur_size: i32,
    horizontal_pass: i32, // 0 or 1 to indicate vertical or horizontal pass
    sigma: f32,           // higher value means more blur
    _p0: f32,
}
@group(1) @binding(0) var<uniform> params: Params;

const pi = 3.14159265;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let num_blur_pixels_per_side = f32(params.blur_size / 2);

    let blur_multiply_vec = select(vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), 0 < params.horizontal_pass);

    // Incremental Gaussian Coefficent Calculation (See GPU Gems 3 pp. 877 - 889)
    var incremental_gaussian: vec3<f32>;
    incremental_gaussian.x = 1.0 / (sqrt(2.0 * pi) * params.sigma);
    incremental_gaussian.y = exp(-0.5 / (params.sigma * params.sigma));
    incremental_gaussian.z = incremental_gaussian.y * incremental_gaussian.y;

    var avg_value = vec4<f32>(0.0);
    var coefficient_sum = 0.0;

    // Take the central sample first...
    avg_value += sample(in.uv) * incremental_gaussian.x;
    coefficient_sum += incremental_gaussian.x;
    incremental_gaussian = vec3<f32>(incremental_gaussian.xy * incremental_gaussian.yz, incremental_gaussian.z);

    // Go through the remaining samples (half on each side of the center)
    let t = texel_size();
    for (var i = 1.0; i <= num_blur_pixels_per_side; i += 1.0) {
        avg_value += sample(in.uv - i * t * blur_multiply_vec) * incremental_gaussian.x;
        avg_value += sample(in.uv + i * t * blur_multiply_vec) * incremental_gaussian.x;
        coefficient_sum += 2.0 * incremental_gaussian.x;
        incremental_gaussian = vec3<f32>(incremental_gaussian.xy * incremental_gaussian.yz, incremental_gaussian.z);
    }

    return avg_value / coefficient_sum;
}
