struct Params {
    forward: vec3<f32>,
    _pad0: f32,
    up: vec3<f32>,
    _pad1: f32,
}

@group(0) @binding(0) var<storage, read> velocity: array<f32>;
@group(0) @binding(1) var<storage, read_write> rotation: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

fn quat_from_to(src: vec3<f32>, dst: vec3<f32>) -> vec4<f32> {
    let d = dot(src, dst);
    if d > 0.999999 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    if d < -0.999999 {
        var axis = cross(src, vec3<f32>(1.0, 0.0, 0.0));
        if dot(axis, axis) < 0.001 {
            axis = cross(src, vec3<f32>(0.0, 1.0, 0.0));
        }
        return vec4<f32>(normalize(axis), 0.0);
    }
    let axis = cross(src, dst);
    return normalize(vec4<f32>(axis, 1.0 + d));
}

fn mat3_to_quat(m: mat3x3<f32>) -> vec4<f32> {
    let trace = m[0][0] + m[1][1] + m[2][2];
    if trace > 0.0 {
        let s = sqrt(trace + 1.0) * 2.0;
        return vec4<f32>(
            (m[1][2] - m[2][1]) / s,
            (m[2][0] - m[0][2]) / s,
            (m[0][1] - m[1][0]) / s,
            0.25 * s,
        );
    }
    if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
        let s = sqrt(1.0 + m[0][0] - m[1][1] - m[2][2]) * 2.0;
        return vec4<f32>(
            0.25 * s,
            (m[1][0] + m[0][1]) / s,
            (m[2][0] + m[0][2]) / s,
            (m[1][2] - m[2][1]) / s,
        );
    }
    if m[1][1] > m[2][2] {
        let s = sqrt(1.0 + m[1][1] - m[0][0] - m[2][2]) * 2.0;
        return vec4<f32>(
            (m[1][0] + m[0][1]) / s,
            0.25 * s,
            (m[2][1] + m[1][2]) / s,
            (m[2][0] - m[0][2]) / s,
        );
    }
    let s = sqrt(1.0 + m[2][2] - m[0][0] - m[1][1]) * 2.0;
    return vec4<f32>(
        (m[2][0] + m[0][2]) / s,
        (m[2][1] + m[1][2]) / s,
        0.25 * s,
        (m[0][1] - m[1][0]) / s,
    );
}

fn write_quat(i: u32, q: vec4<f32>) {
    let ri = i * 4u;
    rotation[ri]      = q.x;
    rotation[ri + 1u] = q.y;
    rotation[ri + 2u] = q.z;
    rotation[ri + 3u] = q.w;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&velocity) / 3u;
    if i >= count { return; }

    let pi = i * 3u;
    let v = vec3<f32>(velocity[pi], velocity[pi + 1u], velocity[pi + 2u]);
    let v_len2 = dot(v, v);
    if v_len2 < 0.000001 { return; }
    let vel_dir = v * inverseSqrt(v_len2);

    let fwd_len2 = dot(params.forward, params.forward);
    if fwd_len2 < 0.000001 { return; }
    let fwd = params.forward * inverseSqrt(fwd_len2);

    let up_param_len2 = dot(params.up, params.up);
    if up_param_len2 < 0.000001 {
        write_quat(i, quat_from_to(fwd, vel_dir));
        return;
    }

    var up_local = params.up - dot(params.up, fwd) * fwd;
    let up_local_len2 = dot(up_local, up_local);
    var up_target = params.up - dot(params.up, vel_dir) * vel_dir;
    let up_target_len2 = dot(up_target, up_target);
    if up_local_len2 < 0.000001 || up_target_len2 < 0.000001 {
        write_quat(i, quat_from_to(fwd, vel_dir));
        return;
    }
    up_local = up_local * inverseSqrt(up_local_len2);
    up_target = up_target * inverseSqrt(up_target_len2);

    let right_local = cross(fwd, up_local);
    let right_target = cross(vel_dir, up_target);

    let local_col = mat3x3<f32>(fwd, up_local, right_local);
    let target_col = mat3x3<f32>(vel_dir, up_target, right_target);
    let r = target_col * transpose(local_col);

    write_quat(i, mat3_to_quat(r));
}
