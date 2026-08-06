struct Params {
    dt: f32,
}

@group(0) @binding(0) var<storage, read_write> age:    array<f32>;
@group(0) @binding(1) var<storage, read_write> life:   array<f32>;
@group(0) @binding(2) var<uniform>             params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count { return; }

    if life[i] <= 0.0 { return; }
    age[i] = age[i] + params.dt;
    if age[i] >= life[i] {
        life[i] = 0.0;
    }
}
