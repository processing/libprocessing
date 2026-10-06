// Toon shader using per-pixel lighting. Based on the glsl
// tutorial from lighthouse 3D:
// http://www.lighthouse3d.com/tutorials/glsl-tutorial/toon-shader-version-ii/
import processing::VertexOutput;

struct Params {
    // Assuming that there is only one directional light: the direction
    // toward it, used to perform per-pixel lighting calculation.
    light_dir: vec3<f32>,
    _p0: f32,
}
@group(3) @binding(0) var<uniform> params: Params;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let vert_normal = normalize(mesh.world_normal);
    let vert_light_dir = normalize(params.light_dir);
    let intensity = max(0.0, dot(vert_light_dir, vert_normal));

    var color: vec4<f32>;
    if (intensity > 0.95) {
        color = vec4<f32>(1.0, 0.5, 0.5, 1.0);
    } else if (intensity > 0.5) {
        color = vec4<f32>(0.6, 0.3, 0.3, 1.0);
    } else if (intensity > 0.25) {
        color = vec4<f32>(0.4, 0.2, 0.2, 1.0);
    } else {
        color = vec4<f32>(0.2, 0.1, 0.1, 1.0);
    }

    return color;
}
