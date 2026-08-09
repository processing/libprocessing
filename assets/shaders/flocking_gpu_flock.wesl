struct Params {
    neighbor_dist: f32,
    separation_dist: f32,
    max_speed: f32,
    max_force: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> steer: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

fn limit(v: vec3<f32>, max_len: f32) -> vec3<f32> {
    let len = length(v);
    if len > max_len { return v * (max_len / len); }
    return v;
}

// Reynolds: steering = desired - velocity
fn steer_toward(desired: vec3<f32>, vel: vec3<f32>) -> vec3<f32> {
    let len = length(desired);
    if len < 1e-6 { return vec3<f32>(0.0); }
    return limit(desired * (params.max_speed / len) - vel, params.max_force);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let pos = vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
    let vel = vec3<f32>(velocity[i * 3u], velocity[i * 3u + 1u], velocity[i * 3u + 2u]);

    var separation = vec3<f32>(0.0);
    var alignment = vec3<f32>(0.0);
    var cohesion = vec3<f32>(0.0);
    var separation_count = 0u;
    var neighbor_count = 0u;

    for (var j = 0u; j < count; j = j + 1u) {
        if j == i { continue; }
        let other = vec3<f32>(position[j * 3u], position[j * 3u + 1u], position[j * 3u + 2u]);
        let d = distance(pos, other);
        if d > 0.0 && d < params.separation_dist {
            // Point away from the neighbor, weighted by closeness
            separation = separation + normalize(pos - other) / d;
            separation_count = separation_count + 1u;
        }
        if d < params.neighbor_dist {
            alignment = alignment
                + vec3<f32>(velocity[j * 3u], velocity[j * 3u + 1u], velocity[j * 3u + 2u]);
            cohesion = cohesion + other;
            neighbor_count = neighbor_count + 1u;
        }
    }

    var force = vec3<f32>(0.0);
    if separation_count > 0u {
        force = force + steer_toward(separation / f32(separation_count), vel) * 1.5;
    }
    if neighbor_count > 0u {
        force = force + steer_toward(alignment, vel);
        force = force + steer_toward(cohesion / f32(neighbor_count) - pos, vel);
    }

    steer[i * 3u] = force.x;
    steer[i * 3u + 1u] = force.y;
    steer[i * 3u + 2u] = force.z;
}
