// Elevated shader
// https://www.shadertoy.com/view/MdX3Rr by inigo quilez

// Created by inigo quilez - iq/2013
// License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.

// Processing port by Raphaël de Courville.
import processing::filter::{resolution, FullscreenVertexOutput};

// Processing specific input
struct Params {
    time: f32,
    _p0: f32,
    mouse: vec2<f32>,
}
@group(1) @binding(0) var<uniform> params: Params;

// ------- Below is the Shadertoy code, in WGSL ----------
// Created by inigo quilez - iq/2013
// License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.

const m = mat3x3<f32>(vec3<f32>(0.00, 0.80, 0.60),
                      vec3<f32>(-0.80, 0.36, -0.48),
                      vec3<f32>(-0.60, -0.48, 0.64));

fn hash(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453123);
}

fn noise3(x: vec3<f32>) -> f32 {
    let p = floor(x);
    var f = fract(x);

    f = f * f * (3.0 - 2.0 * f);

    let n = p.x + p.y * 57.0 + 113.0 * p.z;

    let res = mix(mix(mix(hash(n + 0.0), hash(n + 1.0), f.x),
                      mix(hash(n + 57.0), hash(n + 58.0), f.x), f.y),
                  mix(mix(hash(n + 113.0), hash(n + 114.0), f.x),
                      mix(hash(n + 170.0), hash(n + 171.0), f.x), f.y), f.z);
    return res;
}

fn noised(x: vec2<f32>) -> vec3<f32> {
    let p = floor(x);
    let f = fract(x);

    let u = f * f * (3.0 - 2.0 * f);

    let n = p.x + p.y * 57.0;

    let a = hash(n + 0.0);
    let b = hash(n + 1.0);
    let c = hash(n + 57.0);
    let d = hash(n + 58.0);
    return vec3<f32>(a + (b - a) * u.x + (c - a) * u.y + (a - b - c + d) * u.x * u.y,
                     30.0 * f * f * (f * (f - 2.0) + 1.0) * (vec2<f32>(b - a, c - a) + (a - b - c + d) * u.yx));
}

fn noise2(x: vec2<f32>) -> f32 {
    let p = floor(x);
    var f = fract(x);

    f = f * f * (3.0 - 2.0 * f);

    let n = p.x + p.y * 57.0;

    let res = mix(mix(hash(n + 0.0), hash(n + 1.0), f.x),
                  mix(hash(n + 57.0), hash(n + 58.0), f.x), f.y);

    return res;
}

fn fbm3(p_in: vec3<f32>) -> f32 {
    var p = p_in;
    var f = 0.0;

    f += 0.5000 * noise3(p); p = m * p * 2.02;
    f += 0.2500 * noise3(p); p = m * p * 2.03;
    f += 0.1250 * noise3(p); p = m * p * 2.01;
    f += 0.0625 * noise3(p);

    return f / 0.9375;
}

const m2 = mat2x2<f32>(vec2<f32>(1.6, -1.2), vec2<f32>(1.2, 1.6));

fn fbm2(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    var f = 0.0;

    f += 0.5000 * noise2(p); p = m2 * p * 2.02;
    f += 0.2500 * noise2(p); p = m2 * p * 2.03;
    f += 0.1250 * noise2(p); p = m2 * p * 2.01;
    f += 0.0625 * noise2(p);

    return f / 0.9375;
}

fn terrain(x: vec2<f32>) -> f32 {
    var p = x * 0.003;
    var a = 0.0;
    var b = 1.0;
    var d = vec2<f32>(0.0);
    for (var i = 0; i < 5; i++) {
        let n = noised(p);
        d += n.yz;
        a += b * n.x / (1.0 + dot(d, d));
        b *= 0.5;
        p = mat2x2<f32>(vec2<f32>(1.6, -1.2), vec2<f32>(1.2, 1.6)) * p;
    }

    return 140.0 * a;
}

fn terrain2(x: vec2<f32>) -> f32 {
    var p = x * 0.003;
    var a = 0.0;
    var b = 1.0;
    var d = vec2<f32>(0.0);
    for (var i = 0; i < 14; i++) {
        let n = noised(p);
        d += n.yz;
        a += b * n.x / (1.0 + dot(d, d));
        b *= 0.5;
        p = m2 * p;
    }

    return 140.0 * a;
}

fn map(p: vec3<f32>) -> f32 {
    var h = terrain(p.xz);

    let ss = 0.03;
    let hh = h * ss;
    var fh = fract(hh);
    let ih = floor(hh);
    fh = mix(sqrt(fh), fh, smoothstep(50.0, 140.0, h));
    h = (ih + fh) / ss;

    return p.y - h;
}

fn map2(p: vec3<f32>) -> f32 {
    var h = terrain2(p.xz);

    let ss = 0.03;
    let hh = h * ss;
    var fh = fract(hh);
    let ih = floor(hh);
    fh = mix(sqrt(fh), fh, smoothstep(50.0, 140.0, h));
    h = (ih + fh) / ss;

    return p.y - h;
}

fn jinteresct(ro: vec3<f32>, rd: vec3<f32>, res_t: ptr<function, f32>) -> bool {
    var h = 0.0;
    var t = 0.0;
    for (var j = 0; j < 120; j++) {
        //if( t>2000.0 ) break;

        let p = ro + t * rd;
        if (p.y > 300.0) { break; }
        h = map(p);

        if (h < 0.1) {
            *res_t = t;
            return true;
        }
        t += max(0.1, 0.5 * h);
    }

    if (h < 5.0) {
        *res_t = t;
        return true;
    }
    return false;
}

fn sinteresct(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var res = 1.0;
    var t = 0.0;
    for (var j = 0; j < 50; j++) {
        //if( t>1000.0 ) break;
        let p = ro + t * rd;

        let h = map(p);

        if (h < 0.1) {
            return 0.0;
        }
        res = min(res, 16.0 * h / t);
        t += h;
    }

    return clamp(res, 0.0, 1.0);
}

fn calc_normal(pos: vec3<f32>, t: f32) -> vec3<f32> {
    var e = 0.001;
    e = 0.001 * t;
    let eps = vec3<f32>(e, 0.0, 0.0);
    var nor: vec3<f32>;
    nor.x = map2(pos + eps.xyy) - map2(pos - eps.xyy);
    nor.y = map2(pos + eps.yxy) - map2(pos - eps.yxy);
    nor.z = map2(pos + eps.yyx) - map2(pos - eps.yyx);
    return normalize(nor);
}

fn cam_path(time: f32) -> vec3<f32> {
    let p = 600.0 * vec2<f32>(cos(1.4 + 0.37 * time),
                              cos(3.2 + 0.31 * time));

    return vec3<f32>(p.x, 0.0, p.y);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let i_resolution = resolution();
    // gl_FragCoord has y pointing up; the filter uv has y pointing down
    let frag_coord = vec2<f32>(in.uv.x, 1.0 - in.uv.y) * i_resolution;
    let xy = -1.0 + 2.0 * frag_coord / i_resolution;

    let s = xy * vec2<f32>(1.75, 1.0);

    let time = params.time * 0.15;

    let light1 = normalize(vec3<f32>(0.4, 0.22, 0.6));
    let light2 = vec3<f32>(-0.707, 0.000, -0.707);

    var campos = cam_path(time);
    var camtar = cam_path(time + 3.0);
    campos.y = terrain(campos.xz) + 15.0;
    camtar.y = campos.y * 0.5;

    let roll = 0.1 * cos(0.1 * time);
    let cw = normalize(camtar - campos);
    let cp = vec3<f32>(sin(roll), cos(roll), 0.0);
    let cu = normalize(cross(cw, cp));
    let cv = normalize(cross(cu, cw));
    let rd = normalize(s.x * cu + s.y * cv + 1.6 * cw);

    let sundot = clamp(dot(rd, light1), 0.0, 1.0);
    var col: vec3<f32>;
    var t = 0.0;
    if (!jinteresct(campos, rd, &t)) {
        col = 0.9 * vec3<f32>(0.97, 0.99, 1.0) * (1.0 - 0.3 * rd.y);
        col += 0.2 * vec3<f32>(0.8, 0.7, 0.5) * pow(sundot, 4.0);
    } else {
        let pos = campos + t * rd;

        let nor = calc_normal(pos, t);

        let dif1 = clamp(dot(light1, nor), 0.0, 1.0);
        let dif2 = clamp(0.2 + 0.8 * dot(light2, nor), 0.0, 1.0);
        var sh = 1.0;
        if (dif1 > 0.001) {
            sh = sinteresct(pos + light1 * 20.0, light1);
        }

        var dif1v = vec3<f32>(dif1);
        dif1v *= vec3<f32>(sh, sh * sh * 0.5 + 0.5 * sh, sh * sh);

        let r = noise2(7.0 * pos.xz);

        col = (r * 0.25 + 0.75) * 0.9 * mix(vec3<f32>(0.10, 0.05, 0.03), vec3<f32>(0.13, 0.10, 0.08), clamp(terrain2(vec2<f32>(pos.x, pos.y * 48.0)) / 200.0, 0.0, 1.0));
        col = mix(col, 0.17 * vec3<f32>(0.5, 0.23, 0.04) * (0.50 + 0.50 * r), smoothstep(0.70, 0.9, nor.y));
        col = mix(col, 0.10 * vec3<f32>(0.2, 0.30, 0.00) * (0.25 + 0.75 * r), smoothstep(0.95, 1.0, nor.y));
        col *= 0.75;
        // snow
        let h = smoothstep(55.0, 80.0, pos.y + 25.0 * fbm2(0.01 * pos.xz));
        let e = smoothstep(1.0 - 0.5 * h, 1.0 - 0.1 * h, nor.y);
        let o = 0.3 + 0.7 * smoothstep(0.0, 0.1, nor.x + h * h);
        var sn = h * e * o;
        sn = smoothstep(0.1, 0.9, sn);
        col = mix(col, 0.4 * vec3<f32>(0.6, 0.65, 0.7), sn);

        var brdf = 2.0 * vec3<f32>(0.17, 0.19, 0.20) * clamp(nor.y, 0.0, 1.0);
        brdf += 6.0 * vec3<f32>(1.00, 0.95, 0.80) * dif1v;
        brdf += 2.0 * vec3<f32>(0.20, 0.20, 0.20) * dif2;

        col *= brdf;

        let fo = 1.0 - exp(-pow(0.0015 * t, 1.5));
        let fco = vec3<f32>(0.7) + 0.6 * vec3<f32>(0.8, 0.7, 0.5) * pow(sundot, 4.0);
        col = mix(col, fco, fo);
    }

    col = sqrt(col);

    let uv = xy * 0.5 + 0.5;
    col *= 0.7 + 0.3 * pow(16.0 * uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y), 0.1);

    return vec4<f32>(col, 1.0);
}
