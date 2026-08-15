struct Params {
    sep_distance: f32,
    neighbor_distance: f32,
    weight_separation: f32,
    weight_alignment: f32,
    weight_cohesion: f32,
    max_speed: f32,
    max_force: f32,
    min_speed: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

const TILE: u32 = 64u;
var<workgroup> s_pos: array<vec3<f32>, 64>;
var<workgroup> s_vel: array<vec3<f32>, 64>;

fn limit_mag(v: vec3<f32>, m: f32) -> vec3<f32> {
    let len2 = dot(v, v);
    if len2 > m * m { return v * (m * inverseSqrt(len2)); }
    return v;
}

fn steer_toward(desired: vec3<f32>, vel: vec3<f32>, max_speed: f32, max_force: f32) -> vec3<f32> {
    let m2 = dot(desired, desired);
    if m2 < 0.00000001 { return vec3<f32>(0.0); }
    return limit_mag(desired * (max_speed * inverseSqrt(m2)) - vel, max_force);
}

@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    let alive = i < count;

    let sep_d2 = params.sep_distance * params.sep_distance;
    let neighbor_d2 = params.neighbor_distance * params.neighbor_distance;

    var pos = vec3<f32>(0.0);
    var vel = vec3<f32>(0.0);
    if alive {
        let pi = i * 3u;
        pos = vec3<f32>(position[pi], position[pi + 1u], position[pi + 2u]);
        vel = vec3<f32>(velocity[pi], velocity[pi + 1u], velocity[pi + 2u]);
    }

    var sep_steer = vec3<f32>(0.0);
    var sep_count = 0u;
    var ali_sum = vec3<f32>(0.0);
    var coh_sum = vec3<f32>(0.0);
    var flock_count = 0u;

    let num_tiles = (count + TILE - 1u) / TILE;

    for (var t = 0u; t < num_tiles; t++) {
        let j = t * TILE + lid.x;
        if j < count {
            let pj = j * 3u;
            s_pos[lid.x] = vec3<f32>(position[pj], position[pj + 1u], position[pj + 2u]);
            s_vel[lid.x] = vec3<f32>(velocity[pj], velocity[pj + 1u], velocity[pj + 2u]);
        }
        workgroupBarrier();

        if alive {
            let tile_end = min(TILE, count - t * TILE);
            for (var k = 0u; k < tile_end; k++) {
                let global_j = t * TILE + k;
                if global_j == i { continue; }

                let diff = pos - s_pos[k];
                let d2 = dot(diff, diff);

                if d2 > 0.000001 && d2 < neighbor_d2 {
                    if d2 < sep_d2 {
                        sep_steer += diff / d2;
                        sep_count += 1u;
                    }
                    ali_sum += s_vel[k];
                    coh_sum += diff;
                    flock_count += 1u;
                }
            }
        }
        workgroupBarrier();
    }

    if !alive { return; }

    var force = vec3<f32>(0.0);
    if sep_count > 0u {
        force += steer_toward(sep_steer / f32(sep_count), vel,
                              params.max_speed, params.max_force) * params.weight_separation;
    }
    if flock_count > 0u {
        force += steer_toward(ali_sum / f32(flock_count), vel,
                              params.max_speed, params.max_force) * params.weight_alignment;
        force += steer_toward(-coh_sum / f32(flock_count), vel,
                              params.max_speed, params.max_force) * params.weight_cohesion;
    }

    var new_vel = vel + force;
    let speed2 = dot(new_vel, new_vel);
    let max_speed_sq = params.max_speed * params.max_speed;
    if speed2 > max_speed_sq {
        new_vel = new_vel * (params.max_speed * inverseSqrt(speed2));
    } else if params.min_speed > 0.0 {
        let min_speed_sq = params.min_speed * params.min_speed;
        if speed2 < min_speed_sq && speed2 > 0.0 {
            new_vel = new_vel * sqrt(min_speed_sq / speed2);
        }
    }

    // Steering only: position integration belongs to the INTEGRATE kernel,
    // matching the grid flock's convention.
    let pi = i * 3u;
    velocity[pi]      = new_vel.x;
    velocity[pi + 1u] = new_vel.y;
    velocity[pi + 2u] = new_vel.z;
}
