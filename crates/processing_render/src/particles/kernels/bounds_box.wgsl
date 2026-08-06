struct Params {
    aabb_min: vec3<f32>,
    soft_strength: f32,
    aabb_max: vec3<f32>,
    max_speed: f32,
    mode: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<uniform>             params:   Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let pi = i * 3u;
    var pos = vec3<f32>(position[pi], position[pi + 1u], position[pi + 2u]);
    var vel = vec3<f32>(velocity[pi], velocity[pi + 1u], velocity[pi + 2u]);

    let lo = params.aabb_min;
    let hi = params.aabb_max;
    let size = max(hi - lo, vec3<f32>(0.000001));

    if params.mode == 0u {
        if pos.x > hi.x { pos.x = hi.x; vel.x = min(vel.x, 0.0); }
        if pos.x < lo.x { pos.x = lo.x; vel.x = max(vel.x, 0.0); }
        if pos.y > hi.y { pos.y = hi.y; vel.y = min(vel.y, 0.0); }
        if pos.y < lo.y { pos.y = lo.y; vel.y = max(vel.y, 0.0); }
        if pos.z > hi.z { pos.z = hi.z; vel.z = min(vel.z, 0.0); }
        if pos.z < lo.z { pos.z = lo.z; vel.z = max(vel.z, 0.0); }
    } else if params.mode == 1u {
        if pos.x > hi.x { pos.x = 2.0 * hi.x - pos.x; if vel.x > 0.0 { vel.x = -vel.x; } }
        if pos.x < lo.x { pos.x = 2.0 * lo.x - pos.x; if vel.x < 0.0 { vel.x = -vel.x; } }
        if pos.y > hi.y { pos.y = 2.0 * hi.y - pos.y; if vel.y > 0.0 { vel.y = -vel.y; } }
        if pos.y < lo.y { pos.y = 2.0 * lo.y - pos.y; if vel.y < 0.0 { vel.y = -vel.y; } }
        if pos.z > hi.z { pos.z = 2.0 * hi.z - pos.z; if vel.z > 0.0 { vel.z = -vel.z; } }
        if pos.z < lo.z { pos.z = 2.0 * lo.z - pos.z; if vel.z < 0.0 { vel.z = -vel.z; } }
    } else if params.mode == 2u {
        let rel = pos - lo;
        pos = lo + (rel - size * floor(rel / size));
    } else {
        let over_hi = max(pos - hi, vec3<f32>(0.0));
        let over_lo = max(lo - pos, vec3<f32>(0.0));
        vel = vel - params.soft_strength * (over_hi - over_lo);
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
