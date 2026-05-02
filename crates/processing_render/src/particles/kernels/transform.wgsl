// Built-in transform kernel — applies an affine to each particle's position.
// Order: scale, then rotate around `rotation_axis` by `rotation_angle` radians,
// then translate. Defaults of zero/one behave as identity.
//
// Configure via `compute_set`:
//   translate      : vec3<f32>  — applied last
//   rotation_axis  : vec3<f32>  — need not be normalized
//   rotation_angle : f32        — radians
//   scale          : vec3<f32>  — per-axis scale factor

struct Params {
    translate: vec3<f32>,
    rotation_angle: f32,
    rotation_axis: vec3<f32>,
    scale: vec3<f32>,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;

// Rodrigues' rotation. `axis` must be normalized; `angle` is in radians.
fn rotate(p: vec3<f32>, axis: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return p * c + cross(axis, p) * s + axis * dot(axis, p) * (1.0 - c);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count {
        return;
    }

    var p = vec3<f32>(
        position[i * 3u + 0u],
        position[i * 3u + 1u],
        position[i * 3u + 2u],
    );

    p = p * params.scale;

    let axis_len = length(params.rotation_axis);
    if axis_len > 1.0e-6 && abs(params.rotation_angle) > 1.0e-8 {
        p = rotate(p, params.rotation_axis / axis_len, params.rotation_angle);
    }

    p = p + params.translate;

    position[i * 3u + 0u] = p.x;
    position[i * 3u + 1u] = p.y;
    position[i * 3u + 2u] = p.z;
}
