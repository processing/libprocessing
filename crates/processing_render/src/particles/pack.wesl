// Packs particle attributes into the draw's persistent `GpuMeshInstance`
// output (one slot per particle, slot i = particle i). The renderer
// materializes mesh inputs from it; culling uses the mesh's own bounds.
import bevy_pbr::gpu_instance_batch::gpu_instance::GpuMeshInstance;

struct PackParams {
    count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<storage, read_write> instances: array<GpuMeshInstance>;
@group(0) @binding(1) var<storage, read> position: array<f32>;
@if(HAS_ROTATION)
@group(0) @binding(2) var<storage, read> rotation: array<f32>;
@if(HAS_SCALE)
@group(0) @binding(3) var<storage, read> scale: array<f32>;
@if(HAS_LIFE)
@group(0) @binding(4) var<storage, read> life: array<f32>;
@group(0) @binding(5) var<uniform> params: PackParams;

fn quat_to_basis(q: vec4<f32>) -> mat3x3<f32> {
    let x = q.x; let y = q.y; let z = q.z; let w = q.w;
    let xx = x * x; let yy = y * y; let zz = z * z;
    let xy = x * y; let xz = x * z; let yz = y * z;
    let wx = w * x; let wy = w * y; let wz = w * z;
    return mat3x3<f32>(
        vec3<f32>(1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz),       2.0 * (xz - wy)),
        vec3<f32>(2.0 * (xy - wz),       1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx)),
        vec3<f32>(2.0 * (xz + wy),       2.0 * (yz - wx),       1.0 - 2.0 * (xx + yy)),
    );
}

@compute @workgroup_size(64)
fn pack(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= params.count {
        return;
    }
    let slot = i;

    let pos = vec3<f32>(
        position[i * 3u + 0u],
        position[i * 3u + 1u],
        position[i * 3u + 2u],
    );

    var basis = mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
    );
    @if(HAS_ROTATION) {
        basis = quat_to_basis(vec4<f32>(
            rotation[i * 4u + 0u],
            rotation[i * 4u + 1u],
            rotation[i * 4u + 2u],
            rotation[i * 4u + 3u],
        ));
    }

    var s = vec3<f32>(1.0, 1.0, 1.0);
    @if(HAS_SCALE) {
        s = vec3<f32>(scale[i * 3u + 0u], scale[i * 3u + 1u], scale[i * 3u + 2u]);
    }

    let c0 = basis[0] * s.x;
    let c1 = basis[1] * s.y;
    let c2 = basis[2] * s.z;
    instances[slot].world_from_local = mat3x4<f32>(
        vec4<f32>(c0.x, c1.x, c2.x, pos.x),
        vec4<f32>(c0.y, c1.y, c2.y, pos.y),
        vec4<f32>(c0.z, c1.z, c2.z, pos.z),
    );
    instances[slot].tag = i;
    // A dead particle (life <= 0) is an inactive slot and draws nothing.
    instances[slot].is_active = 1u;
    @if(HAS_LIFE) {
        instances[slot].is_active = select(0u, 1u, life[i] > 0.0);
    }
}
