struct Params {
    mode: u32,
    count: u32,
}

const BLOCK: u32 = 256u;

@group(0) @binding(0) var<storage, read>       input:  array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<uniform>             params: Params;

var<workgroup> sdata: array<f32, 256>;

fn ident(mode: u32) -> f32 {
    if mode == 1u { return 3.4e38; }
    if mode == 2u { return -3.4e38; }
    return 0.0;
}

fn combine(a: f32, b: f32, mode: u32) -> f32 {
    if mode == 1u { return min(a, b); }
    if mode == 2u { return max(a, b); }
    return a + b;
}

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    var v = ident(params.mode);
    if gid.x < params.count {
        v = input[gid.x];
    }
    sdata[lid.x] = v;
    workgroupBarrier();

    var stride = BLOCK / 2u;
    loop {
        if stride == 0u { break; }
        if lid.x < stride {
            sdata[lid.x] = combine(sdata[lid.x], sdata[lid.x + stride], params.mode);
        }
        workgroupBarrier();
        stride = stride >> 1u;
    }

    if lid.x == 0u {
        output[wid.x] = sdata[0];
    }
}
