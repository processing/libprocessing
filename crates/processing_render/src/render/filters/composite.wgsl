// Composite (blend) filter: combines the destination surface (operand A, sampled
// from the screen texture) with a source input texture (operand B, `src`) using a
// blend mode. This is the shader-based composite the fixed-function blend path
// cannot express exactly (MULTIPLY, DIFFERENCE, EXCLUSION, MASK).
//
// `mode` matches the `BlendMode` enum discriminants:
//   0 BLEND, 1 ADD, 2 SUBTRACT, 3 DARKEST, 4 LIGHTEST, 5 DIFFERENCE,
//   6 EXCLUSION, 7 MULTIPLY, 8 SCREEN, 9 REPLACE (copy), 10 MASK.
//
// `src_rect`/`dst_rect` are normalized (uv) rectangles (x0, y0, x1, y1). Only
// pixels inside `dst_rect` are affected; the source is sampled across `src_rect`,
// so a smaller/larger dst_rect scales the source into place.
import processing::filter::{sample, sample_texture, FullscreenVertexOutput};

struct CompositeParams {
    src_rect: vec4<f32>,
    dst_rect: vec4<f32>,
    mode: u32,
    opacity: f32,
    _p0: f32,
    _p1: f32,
}

@group(1) @binding(0) var src: texture_2d<f32>;
@group(1) @binding(1) var<uniform> composite_params: CompositeParams;

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// Per-channel blend of destination `d` and source `s` for the non-alpha-defined
// modes (BLEND/REPLACE/MASK are handled in `composite`).
fn blend_rgb(d: vec3<f32>, s: vec3<f32>, mode: u32) -> vec3<f32> {
    switch mode {
        case 1u: { return d + s; }                        // ADD
        case 2u: { return d - s; }                        // SUBTRACT
        case 3u: { return min(d, s); }                    // DARKEST
        case 4u: { return max(d, s); }                    // LIGHTEST
        case 5u: { return abs(d - s); }                   // DIFFERENCE
        case 6u: { return d + s - 2.0 * d * s; }          // EXCLUSION
        case 7u: { return d * s; }                        // MULTIPLY
        case 8u: { return 1.0 - (1.0 - d) * (1.0 - s); }  // SCREEN
        default: { return s; }
    }
}

fn composite(d: vec4<f32>, s: vec4<f32>, mode: u32, opacity: f32) -> vec4<f32> {
    // BLEND: straight-alpha source-over.
    if (mode == 0u) {
        let a = s.a * opacity;
        let out_a = a + d.a * (1.0 - a);
        var rgb = vec3<f32>(0.0);
        if (out_a > 0.0) {
            rgb = (s.rgb * a + d.rgb * d.a * (1.0 - a)) / out_a;
        }
        return vec4<f32>(rgb, out_a);
    }
    // REPLACE / copy.
    if (mode == 9u) {
        return mix(d, s, opacity);
    }
    // MASK: keep destination color, take alpha from the source's luminance.
    if (mode == 10u) {
        return vec4<f32>(d.rgb, d.a * mix(1.0, luminance(s.rgb), opacity));
    }
    // Remaining modes define a blended rgb, applied over the destination scaled
    // by the (opacity-weighted) source alpha.
    let a = s.a * opacity;
    let blended = clamp(blend_rgb(d.rgb, s.rgb, mode), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(mix(d.rgb, blended, a), d.a);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let d = sample(in.uv);

    let dmin = composite_params.dst_rect.xy;
    let dmax = composite_params.dst_rect.zw;
    if (in.uv.x < dmin.x || in.uv.y < dmin.y || in.uv.x > dmax.x || in.uv.y > dmax.y) {
        return d;
    }

    let local = (in.uv - dmin) / max(dmax - dmin, vec2<f32>(1e-6));
    let suv = mix(composite_params.src_rect.xy, composite_params.src_rect.zw, local);
    let s = sample_texture(src, suv);
    return composite(d, s, composite_params.mode, composite_params.opacity);
}
