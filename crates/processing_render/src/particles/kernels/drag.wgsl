struct Params {
    damping: f32,
    max_speed: f32,
}

@group(0) @binding(0) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&velocity) / 3u;
    if i >= count { return; }

    let pi = i * 3u;
    let v = vec3<f32>(velocity[pi], velocity[pi + 1u], velocity[pi + 2u]);
    let k = clamp(params.damping, 0.0, 1.0);
    var new_v = v * (1.0 - k);
    if params.max_speed > 0.0 {
        let speed2 = dot(new_v, new_v);
        let cap2 = params.max_speed * params.max_speed;
        if speed2 > cap2 {
            new_v = new_v * (params.max_speed * inverseSqrt(speed2));
        }
    }
    velocity[pi]      = new_v.x;
    velocity[pi + 1u] = new_v.y;
    velocity[pi + 2u] = new_v.z;
}
