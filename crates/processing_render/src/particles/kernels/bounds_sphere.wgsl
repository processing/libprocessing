struct Params {
    center: vec3<f32>,
    radius: f32,
    max_speed: f32,
    soft_strength: f32,
    mode: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let pi = i * 3u;
    var pos = vec3<f32>(position[pi], position[pi + 1u], position[pi + 2u]);
    var vel = vec3<f32>(velocity[pi], velocity[pi + 1u], velocity[pi + 2u]);

    let offset = pos - params.center;
    let d2 = dot(offset, offset);
    let r2 = params.radius * params.radius;

    if d2 > r2 && d2 > 0.000001 {
        let d = sqrt(d2);
        let normal = offset / d;

        if params.mode == 0u {
            pos = params.center + normal * params.radius;
            let outward = max(0.0, dot(vel, normal));
            vel = vel - normal * outward;
        } else if params.mode == 1u {
            let overshoot = d - params.radius;
            pos = pos - normal * (2.0 * overshoot);
            let outward = max(0.0, dot(vel, normal));
            vel = vel - normal * (2.0 * outward);
        } else if params.mode == 3u {
            vel = vel - normal * (params.soft_strength * (d - params.radius));
        }
    }

    if params.max_speed > 0.0 {
        let speed2 = dot(vel, vel);
        let cap2 = params.max_speed * params.max_speed;
        if speed2 > cap2 {
            vel = vel * (params.max_speed * inverseSqrt(speed2));
        }
    }

    position[pi]      = pos.x;
    position[pi + 1u] = pos.y;
    position[pi + 2u] = pos.z;
    velocity[pi]      = vel.x;
    velocity[pi + 1u] = vel.y;
    velocity[pi + 2u] = vel.z;
}
