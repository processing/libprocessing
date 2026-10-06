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

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // gl_FragCoord has y pointing up; the filter uv has y pointing down
    let frag_coord = vec2<f32>(in.uv.x, 1.0 - in.uv.y) * resolution();
    let p = -1.0 + 2.0 * frag_coord / resolution();
    let a = params.time * 40.0;
    let g = 1.0 / 40.0;
    var e = 400.0 * (p.x * 0.5 + 0.5);
    var f = 400.0 * (p.y * 0.5 + 0.5);
    var i = 200.0 + sin(e * g + a / 150.0) * 20.0;
    var d = 200.0 + cos(f * g / 2.0) * 18.0 + cos(e * g) * 7.0;
    let r = sqrt(pow(i - e, 2.0) + pow(d - f, 2.0));
    let q = f / r;
    e = (r * cos(q)) - a / 2.0;
    f = (r * sin(q)) - a / 2.0;
    d = sin(e * g) * 176.0 + sin(e * g) * 164.0 + r;
    var h = ((f + d) + a / 2.0) * g;
    i = cos(h + r * p.x / 1.3) * (e + e + a) + cos(q * g * 6.0) * (r + h / 3.0);
    h = sin(f * g) * 144.0 - sin(e * g) * 212.0 * p.x;
    h = (h + (f - e) * q + sin(r - (a + h) / 7.0) * 10.0 + i / 4.0) * g;
    i += cos(h * 2.3 * sin(a / 350.0 - q)) * 184.0 * sin(q - (r * 4.3 + a / 12.0) * g) + tan(r * g + h) * 184.0 * cos(r * g + h);
    i = mod_f(i / 5.6, 256.0) / 64.0;
    if (i < 0.0) { i += 4.0; }
    if (i >= 2.0) { i = 4.0 - i; }
    d = r / 350.0;
    d += sin(d * d * 8.0) * 0.52;
    f = (sin(a * g) + 1.0) / 2.0;
    return vec4<f32>(vec3<f32>(f * i / 1.6, i / 2.0 + d / 13.0, i) * d * p.x + vec3<f32>(i / 1.3 + d / 8.0, i / 2.0 + d / 18.0, i) * d * (1.0 - p.x), 1.0);
}
