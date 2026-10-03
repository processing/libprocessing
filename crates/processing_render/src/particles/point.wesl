import bevy_render::view::View;

@group(0) @binding(0) var<uniform> view: View;
@group(1) @binding(0) var<storage, read> positions: array<f32>;
@if(HAS_COLORS)
@group(1) @binding(1) var<storage, read> colors: array<f32>;
@if(HAS_NORMALS)
@group(1) @binding(2) var<storage, read> normals: array<f32>;

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
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    @if(HAS_COLORS) {
        out.color = vec4<f32>(
            colors[i * 4u], colors[i * 4u + 1u], colors[i * 4u + 2u], colors[i * 4u + 3u]);
    }
    out.normal = vec3<f32>(0.0, 0.0, 0.0);
    @if(HAS_NORMALS) {
        out.normal = vec3<f32>(normals[i * 3u], normals[i * 3u + 1u], normals[i * 3u + 2u]);
    }
    return out;
}

@fragment
fn fragment(frag: VertexOutput) -> @location(0) vec4<f32> {
    var rgb = frag.color.rgb;
    @if(SHADED) {
        var normal = normalize(cross(dpdx(frag.world), dpdy(frag.world)));
        @if(HAS_NORMALS) {
            normal = normalize(frag.normal);
        }
        let light = normalize(vec3<f32>(0.4, 0.85, 0.35));
        let diffuse = abs(dot(normal, light));
        rgb = rgb * (0.2 + 0.8 * diffuse);
    }
    return vec4<f32>(rgb, frag.color.a);
}
