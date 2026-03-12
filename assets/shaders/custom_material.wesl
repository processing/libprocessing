import { lygia::generative::snoise::snoise33 };
import { processing::{globals, VertexOutput} };

@group(3) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = mesh.world_position.xyz * 0.02 + globals.time * 0.1;
    let n = snoise33(p) * 0.5 + 0.5;

    let light_dir = normalize(vec3(0.5, 1.0, 0.3));
    let diffuse = max(dot(normalize(mesh.world_normal), light_dir), 0.15);

    return vec4(n * diffuse, 1.0) * color;
}
