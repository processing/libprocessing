#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;
@group(1) @binding(0) var<storage, read> positions: array<f32>;
#ifdef HAS_COLORS
@group(1) @binding(1) var<storage, read> colors: array<f32>;
#endif
#ifdef HAS_NORMALS
@group(1) @binding(2) var<storage, read> normals: array<f32>;
#endif

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let i = vertex_index;
    let world = vec3<f32>(positions[i * 3u], positions[i * 3u + 1u], positions[i * 3u + 2u]);

    var out: VertexOutput;
    out.world = world;
    out.clip_position = view.clip_from_world * vec4<f32>(world, 1.0);
#ifdef HAS_COLORS
    out.color = vec4<f32>(
        colors[i * 4u], colors[i * 4u + 1u], colors[i * 4u + 2u], colors[i * 4u + 3u]);
#else
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
#endif
#ifdef HAS_NORMALS
    out.normal = vec3<f32>(normals[i * 3u], normals[i * 3u + 1u], normals[i * 3u + 2u]);
#else
    out.normal = vec3<f32>(0.0, 0.0, 0.0);
#endif
    return out;
}

@fragment
fn fragment(frag: VertexOutput) -> @location(0) vec4<f32> {
    var rgb = frag.color.rgb;
#ifdef SHADED
    #ifdef HAS_NORMALS
    let normal = normalize(frag.normal);
    #else
    let normal = normalize(cross(dpdx(frag.world), dpdy(frag.world)));
    #endif
    let light = normalize(vec3<f32>(0.4, 0.85, 0.35));
    let diffuse = abs(dot(normal, light));
    rgb = rgb * (0.2 + 0.8 * diffuse);
#endif
    return vec4<f32>(rgb, frag.color.a);
}
