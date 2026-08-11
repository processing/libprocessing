const BLOCK: u32 = 256u;

@group(0) @binding(0) var<storage, read_write> data: array<u32>;
@group(0) @binding(1) var<storage, read_write> block_sums: array<u32>;

var<workgroup> tmp: array<u32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    let n = arrayLength(&data);
    let g = gid.x;
    let l = lid.x;

    var v: u32 = 0u;
    if g < n { v = data[g]; }
    tmp[l] = v;
    workgroupBarrier();

    for (var offset: u32 = 1u; offset < BLOCK; offset = offset << 1u) {
        var add: u32 = 0u;
        if l >= offset { add = tmp[l - offset]; }
        workgroupBarrier();
        tmp[l] = tmp[l] + add;
        workgroupBarrier();
    }

    if g < n { data[g] = tmp[l] - v; }

    if l == 0u { block_sums[wid.x] = tmp[BLOCK - 1u]; }
}
