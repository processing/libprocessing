// Copyright (C) 2007 Dave Griffiths
// Licence: GPLv2 (see COPYING)
// Fluxus Shader Library
// ---------------------
// Glossy Specular Reflection Shader
// A more controllable version of blinn shading,
// Useful for ceramic or fluids - from Advanced
// Renderman, thanks to Larry Gritz
import processing::VertexOutput;

struct Params {
    ambient_colour: vec3<f32>,
    ambient_intensity: f32,
    diffuse_colour: vec3<f32>,
    diffuse_intensity: f32,
    specular_colour: vec3<f32>,
    specular_intensity: f32,
    // the light and the eye, in world space (the vertex stage of the
    // GLSL version computed these per vertex)
    light_position: vec3<f32>,
    roughness: f32,
    camera_position: vec3<f32>,
    sharpness: f32,
}
@group(3) @binding(0) var<uniform> params: Params;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let w = 0.18 * (1.0 - params.sharpness);

    let p = mesh.world_position.xyz;
    let l = normalize(params.light_position - p);
    let n = normalize(mesh.world_normal);
    let v = normalize(params.camera_position - p);
    let h = normalize(l + v);

    let diffuse = dot(l, n);
    let specular = smoothstep(0.72 - w, 0.72 + w, pow(max(0.0, dot(n, h)), 1.0 / params.roughness));

    return vec4<f32>(params.ambient_colour * params.ambient_intensity +
                     params.diffuse_colour * diffuse * params.diffuse_intensity +
                     params.specular_colour * specular * params.specular_intensity, 1.0);
}
