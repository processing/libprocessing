// Copies one rectangle of the surface onto another; rects are normalized (x0, y0, x1, y1).
import processing::filter::{sample, FullscreenVertexOutput};

struct CopyRegionParams {
    src_rect: vec4<f32>,
    dst_rect: vec4<f32>,
}

@group(1) @binding(0) var<uniform> copy_region: CopyRegionParams;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let dmin = copy_region.dst_rect.xy;
    let dmax = copy_region.dst_rect.zw;
    if (in.uv.x < dmin.x || in.uv.y < dmin.y || in.uv.x > dmax.x || in.uv.y > dmax.y) {
        return sample(in.uv);
    }
    let local = (in.uv - dmin) / max(dmax - dmin, vec2<f32>(1e-6));
    return sample(mix(copy_region.src_rect.xy, copy_region.src_rect.zw, local));
}
