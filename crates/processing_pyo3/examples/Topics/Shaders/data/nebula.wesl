// NEBULA - CoffeeBreakStudios.com (CBS)
// Work in progress...
//
// 3148.26: Switched from classic to simplex noise
// 3148.27: Reduced number of stars
// 3249.0:  Switched to fast computed 3D noise. Less quality but ~ 2x faster
// 3249.5:  Removed use of random number generator to gain performance
// 3265.0:  Added rotation: glsl.heroku.com/e#3005.1
import processing::filter::{resolution, FullscreenVertexOutput};

struct Params {
    time: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}
@group(1) @binding(0) var<uniform> params: Params;

// GLSL mod(): floored modulo (WGSL `%` truncates)
fn mod_f(x: f32, y: f32) -> f32 { return x - y * floor(x / y); }
fn mod3(x: vec3<f32>, y: vec3<f32>) -> vec3<f32> { return x - y * floor(x / y); }
fn mod4(x: vec4<f32>, y: vec4<f32>) -> vec4<f32> { return x - y * floor(x / y); }

//Utility functions

fn rotate(point: vec2<f32>, rads: f32) -> vec2<f32> {
    let cs = cos(rads);
    let sn = sin(rads);
    return point * mat2x2<f32>(vec2<f32>(cs, -sn), vec2<f32>(sn, cs));
}

fn randomizer4(x: vec4<f32>) -> vec4<f32> {
    var z = mod4(x, vec4<f32>(5612.0));
    z = mod4(z, vec4<f32>(3.1415927 * 2.0));
    return fract(cos(z) * vec4<f32>(56812.5453));
}

// Fast computed noise
// http://www.gamedev.net/topic/502913-fast-computed-noise/

const A = 1.0;
const B = 57.0;
const C = 113.0;
const ABC = vec3<f32>(A, B, C);
const A3 = vec4<f32>(0.0, B, C, C + B);
const A4 = vec4<f32>(A, A + B, C + A, C + A + B);

fn cnoise4(xx: vec3<f32>) -> f32 {
    let x = mod3(xx + 32768.0, vec3<f32>(65536.0));
    let ix = floor(x);
    let fx = fract(x);
    let wx = fx * fx * (3.0 - 2.0 * fx);
    let nn = dot(ix, ABC);

    let n1 = nn + A3;
    let n2 = nn + A4;
    let r1 = randomizer4(n1);
    let r2 = randomizer4(n2);
    let r = mix(r1, r2, wx.x);
    let re = mix(mix(r.x, r.y, wx.y), mix(r.z, r.w, wx.y), wx.z);

    return 1.0 - 2.0 * re;
}

fn surface3(coord: vec3<f32>, frequency: f32) -> f32 {
    var n = 0.0;

    n += 1.0    * abs(cnoise4(coord * frequency));
    n += 0.5    * abs(cnoise4(coord * frequency * 2.0));
    n += 0.25   * abs(cnoise4(coord * frequency * 4.0));
    n += 0.125  * abs(cnoise4(coord * frequency * 8.0));
    n += 0.0625 * abs(cnoise4(coord * frequency * 16.0));

    return n;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let time = params.time;
    // gl_FragCoord has y pointing up; the filter uv has y pointing down
    let frag_coord = vec2<f32>(in.uv.x, 1.0 - in.uv.y) * resolution();

    let rads = radians(time * 3.15);
    var position = frag_coord / resolution();
    position += rotate(position, rads);
    let tilt = mat3x3<f32>(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.8, 0.6), vec3<f32>(0.0, -0.6, 0.8));
    let n = surface3(vec3<f32>(position * sin(time * 0.1), time * 0.05) * tilt, 0.9);
    let n2 = surface3(vec3<f32>(position * cos(time * 0.1), time * 0.04) * tilt, 0.8);
    let lum = length(n);
    let lum2 = length(n2);

    let tc = pow(vec3<f32>(1.0 - lum), vec3<f32>(sin(position.x) + cos(time) + 4.0, 8.0 + sin(time) + 4.0, 8.0));
    let tc2 = pow(vec3<f32>(1.1 - lum2), vec3<f32>(5.0, position.y + cos(time) + 7.0, sin(position.x) + sin(time) + 2.0));
    var curr_color = (tc * 0.8) + (tc2 * 0.5);

    //Let's draw some stars

    let scale = sin(0.3 * time) + 5.0;
    let position2 = ((frag_coord / resolution()) - 0.5) * scale;
    var gradient = 0.0;
    var centered_coord = position2 - vec2<f32>(sin(time * 0.1), sin(time * 0.1));
    centered_coord = rotate(centered_coord, rads);

    for (var i = 1.0; i <= 60.0; i += 1.0) {
        let star_pos = vec2<f32>(sin(i) * 250.0, sin(i * i * i) * 250.0);
        let z = mod_f(i * i - 10.0 * time, 256.0);
        let fade = (256.0 - z) / 256.0;
        let blob_coord = star_pos / z;
        gradient += ((fade / 384.0) / pow(length(centered_coord - blob_coord), 1.5)) * fade;
    }

    curr_color += gradient;

    return vec4<f32>(curr_color, 1.0);
}
