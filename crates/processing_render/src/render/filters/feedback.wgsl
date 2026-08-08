// Feedback filter: samples the surface's *previous* frame (operand A) with a
// zoom/rotate/offset transform and a per-frame decay, writing it back. On a
// graphics context that isn't cleared each frame, applying this at the start of
// draw() and then drawing new content on top produces feedback trails
// (TouchDesigner Feedback-TOP style). This is just a filter whose input is the
// target's own history.
import processing::filter::{sample, FullscreenVertexOutput};

struct FeedbackParams {
    offset: vec2<f32>,
    zoom: f32,
    angle: f32,
    decay: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}

@group(1) @binding(0) var<uniform> feedback: FeedbackParams;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    var uv = in.uv - center;

    // Rotate then inverse-zoom around the center (sampling the previous frame).
    let s = sin(feedback.angle);
    let c = cos(feedback.angle);
    uv = mat2x2<f32>(c, -s, s, c) * uv;
    uv = uv / max(feedback.zoom, 0.0001);

    uv = uv + center - feedback.offset;
    return sample(uv) * feedback.decay;
}
