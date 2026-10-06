// Port of MeshTweening's vert.glsl / frag.glsl: blends each vertex between its
// position and its "tweened" attribute. Target shader for the custom
// per-vertex attribute gap; the vertex input layout is not yet exposed.
import { processing::{globals, VertexOutput} };

@group(3) @binding(0) var<uniform> tween: f32;

struct TweenVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tweened: vec3<f32>,
};

@vertex
fn vertex(v: TweenVertex) -> VertexOutput {
    var out: VertexOutput;
    let p = (1.0 - tween) * v.position + tween * v.tweened;
    out.position = globals.view_proj * globals.model * vec4(p, 1.0);
    out.color = v.color;
    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    return mesh.color;
}
