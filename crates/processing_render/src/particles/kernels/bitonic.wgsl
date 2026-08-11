struct Params {
    k: u32,
    j: u32,
}

@group(0) @binding(0) var<storage, read_write> keys:    array<f32>;
@group(0) @binding(1) var<storage, read_write> payload: array<u32>;
@group(0) @binding(2) var<uniform>             params:  Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&keys);
    if i >= n { return; }

    let partner = i ^ params.j;
    if partner <= i || partner >= n { return; }

    let up = (i & params.k) == 0u;
    let ki = keys[i];
    let kp = keys[partner];
    if (ki > kp) == up {
        keys[i] = kp;
        keys[partner] = ki;
        let pi = payload[i];
        payload[i] = payload[partner];
        payload[partner] = pi;
    }
}
