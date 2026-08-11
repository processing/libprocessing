import processing::particles::{cell_coords, cell_index};

struct FlockParams {
    sep_distance: f32,
    neighbor_distance: f32,
    weight_separation: f32,
    weight_alignment: f32,
    weight_cohesion: f32,
    max_speed: f32,
    max_force: f32,
    min_speed: f32,
}

struct GridParams {
    grid_min: vec3<f32>,
    cell_size: f32,
    dims_x: u32,
    dims_y: u32,
    dims_z: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read>       position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read>       offsets:  array<u32>;
@group(0) @binding(3) var<storage, read>       sorted:   array<u32>;
@group(0) @binding(4) var<uniform>             fp:       FlockParams;
@group(0) @binding(5) var<uniform>             gp:       GridParams;

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

fn load_pos(i: u32) -> vec3<f32> {
    return vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
}

fn load_vel(i: u32) -> vec3<f32> {
    return vec3<f32>(velocity[i * 3u], velocity[i * 3u + 1u], velocity[i * 3u + 2u]);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let pos = load_pos(i);
    let vel = load_vel(i);

    let sep_d2 = fp.sep_distance * fp.sep_distance;
    let neighbor_d2 = fp.neighbor_distance * fp.neighbor_distance;

    let dims = vec3<u32>(gp.dims_x, gp.dims_y, gp.dims_z);
    let base = cell_coords(pos, gp.grid_min, gp.cell_size, dims);
    let bx = base.x;
    let by = base.y;
    let bz = base.z;

    var sep_steer = vec3<f32>(0.0);
    var sep_count = 0u;
    var ali_sum = vec3<f32>(0.0);
    var coh_sum = vec3<f32>(0.0);
    var flock_count = 0u;

    for (var dz = -1; dz <= 1; dz++) {
        let cz = bz + dz;
        if cz < 0 || cz >= i32(gp.dims_z) { continue; }
        for (var dy = -1; dy <= 1; dy++) {
            let cy = by + dy;
            if cy < 0 || cy >= i32(gp.dims_y) { continue; }
            for (var dx = -1; dx <= 1; dx++) {
                let cx = bx + dx;
                if cx < 0 || cx >= i32(gp.dims_x) { continue; }

                let cell = cell_index(vec3<u32>(u32(cx), u32(cy), u32(cz)), dims);
                let start = offsets[cell];
                let end = offsets[cell + 1u];
                for (var s = start; s < end; s++) {
                    let j = sorted[s];
                    if j == i { continue; }

                    let diff = pos - load_pos(j);
                    let d2 = dot(diff, diff);
                    if d2 > 0.000001 && d2 < neighbor_d2 {
                        if d2 < sep_d2 {
                            sep_steer += diff / d2;
                            sep_count += 1u;
                        }
                        ali_sum += load_vel(j);
                        coh_sum += diff;
                        flock_count += 1u;
                    }
                }
            }
        }
    }

    var force = vec3<f32>(0.0);
    if sep_count > 0u {
        force += steer_toward(sep_steer / f32(sep_count), vel,
                              fp.max_speed, fp.max_force) * fp.weight_separation;
    }
    if flock_count > 0u {
        force += steer_toward(ali_sum / f32(flock_count), vel,
                              fp.max_speed, fp.max_force) * fp.weight_alignment;
        force += steer_toward(-coh_sum / f32(flock_count), vel,
                              fp.max_speed, fp.max_force) * fp.weight_cohesion;
    }

    var new_vel = vel + force;
    let speed2 = dot(new_vel, new_vel);
    let max_speed_sq = fp.max_speed * fp.max_speed;
    if speed2 > max_speed_sq {
        new_vel = new_vel * (fp.max_speed * inverseSqrt(speed2));
    } else if fp.min_speed > 0.0 {
        let min_speed_sq = fp.min_speed * fp.min_speed;
        if speed2 < min_speed_sq && speed2 > 0.0 {
            new_vel = new_vel * sqrt(min_speed_sq / speed2);
        }
    }

    velocity[i * 3u]      = new_vel.x;
    velocity[i * 3u + 1u] = new_vel.y;
    velocity[i * 3u + 2u] = new_vel.z;
}
