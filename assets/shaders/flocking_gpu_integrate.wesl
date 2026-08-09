struct Params {
    dt: f32,
    max_speed: f32,
    bound: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> steer: array<f32>;
@group(0) @binding(3) var<storage, read_write> rotation: array<f32>;
@group(0) @binding(4) var<uniform> params: Params;

// shortest-arc quaternion rotating the mesh's +Z axis onto dir
fn quat_z_to(dir: vec3<f32>) -> vec4<f32> {
    let z = vec3<f32>(0.0, 0.0, 1.0);
    let d = dot(z, dir);
    if d < -0.9999 { return vec4<f32>(0.0, 1.0, 0.0, 0.0); }
    return normalize(vec4<f32>(cross(z, dir), 1.0 + d));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    var pos = vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
    var vel = vec3<f32>(velocity[i * 3u], velocity[i * 3u + 1u], velocity[i * 3u + 2u]);
    let force = vec3<f32>(steer[i * 3u], steer[i * 3u + 1u], steer[i * 3u + 2u]);

    vel = vel + force * params.dt;
    let speed = length(vel);
    if speed > params.max_speed { vel = vel * (params.max_speed / speed); }
    pos = pos + vel * params.dt;

    // wrap into [-bound, bound]: ((p + b) mod 2b + 2b) mod 2b - b
    let span = 2.0 * params.bound;
    pos = ((pos + params.bound) % span + span) % span - params.bound;

    position[i * 3u] = pos.x;
    position[i * 3u + 1u] = pos.y;
    position[i * 3u + 2u] = pos.z;
    velocity[i * 3u] = vel.x;
    velocity[i * 3u + 1u] = vel.y;
    velocity[i * 3u + 2u] = vel.z;

    if speed > 1e-6 {
        let q = quat_z_to(vel / speed);
        rotation[i * 4u] = q.x;
        rotation[i * 4u + 1u] = q.y;
        rotation[i * 4u + 2u] = q.z;
        rotation[i * 4u + 3u] = q.w;
    }
}
