//---------------------------------------------------------
// Display endless moving background using a tile texture.
// Contributed by martiSteiger
//---------------------------------------------------------
import processing::filter::{sample_texture, resolution, FullscreenVertexOutput};

struct Params {
    time: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}
@group(1) @binding(0) var<uniform> params: Params;
@group(1) @binding(1) var tile_image: texture_2d<f32>;

const TILES_COUNT_X = 4.0;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let res = resolution();
    // gl_FragCoord has y pointing up; the filter uv has y pointing down
    let frag_coord = vec2<f32>(in.uv.x, 1.0 - in.uv.y) * res;
    let pos = frag_coord - vec2<f32>(4.0 * params.time);
    let p = (res - TILES_COUNT_X * pos) / res.x;
    // fract() wraps the texture like textureWrap(REPEAT)
    let col = sample_texture(tile_image, fract(p)).xyz;
    return vec4<f32>(col, 1.0);
}
