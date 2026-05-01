// Per-particle color material for [`Field`] rasterization.
//
// Reads `mesh.tag` (written by the pack pass as the per-particle slot index)
// and looks up a per-particle color from a storage buffer. Unlit — outputs the
// looked-up color directly.

#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<storage, read> particle_colors: array<vec4<f32>>;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let tag = mesh_functions::get_tag(vertex.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0),
    );
    out.clip_position = position_world_to_clip(world_position.xyz);
    out.color = particle_colors[tag];
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
