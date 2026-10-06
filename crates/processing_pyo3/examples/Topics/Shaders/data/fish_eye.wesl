// Inspired by the "Angular Fisheye à la Bourke" sketch from
// Jonathan Cremieux, as shown in the OpenProcessing website:
// http://openprocessing.org/visuals/?visualID=12140
// Using the inverse transform of the angular fisheye as
// explained in Paul Bourke's website:
// http://paulbourke.net/miscellaneous/domefisheye/fisheye/
import processing::filter::{sample, FullscreenVertexOutput};

struct Params {
    aperture: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}
@group(1) @binding(0) var<uniform> params: Params;

const PI = 3.1415926535;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let aperture_half = 0.5 * params.aperture * (PI / 180.0);

    // This factor ajusts the coordinates in the case that
    // the aperture angle is less than 180 degrees, in which
    // case the area displayed is not the entire half-sphere.
    let max_factor = sin(aperture_half);

    // The canvas texture always covers the full 0..1 uv range here.
    let pos = 2.0 * in.uv - 1.0;

    let l = length(pos);
    if (l > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        let x = max_factor * pos.x;
        let y = max_factor * pos.y;

        let n = length(vec2<f32>(x, y));

        let z = sqrt(1.0 - n * n);

        let r = atan2(n, z) / PI;

        let phi = atan2(y, x);

        let u = r * cos(phi) + 0.5;
        let v = r * sin(phi) + 0.5;

        return sample(vec2<f32>(u, v));
    }
}
