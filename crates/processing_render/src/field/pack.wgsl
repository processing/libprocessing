// Pack pass — bridges libprocessing Field buffers into the upstream
// per-instance MeshInputUniform / MeshCullingData slots reserved by
// `GpuBatchedMesh3d`.
//
// Specialized via shader_defs:
//   HAS_ROTATION — bind a `rotation` buffer (Float4 quaternion `xyzw`)
//   HAS_SCALE    — bind a `scale` buffer (Float3)
//   HAS_DEAD     — bind a `dead` buffer (Float, 0 = alive, non-zero = dead)
//
// Buffer formats (CPU-tightly-packed):
//   position : 12 bytes per particle (Float3)
//   rotation : 16 bytes per particle (Float4 quat)
//   scale    : 12 bytes per particle (Float3)
//   dead     :  4 bytes per particle (Float)

struct MeshInput {
    world_from_local: mat3x4<f32>,
    lightmap_uv_rect: vec2<u32>,
    flags: u32,
    previous_input_index: u32,
    first_vertex_index: u32,
    first_index_index: u32,
    index_count: u32,
    current_skin_index: u32,
    material_and_lightmap_bind_group_slot: u32,
    timestamp: u32,
    tag: u32,
    morph_descriptor_index: u32,
}

struct MeshCullingData {
    aabb_center: vec3<f32>,
    _pad: f32,
    aabb_half_extents: vec3<f32>,
    dead: f32,
}

struct PackParams {
    base_input_index: u32,
    count: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read_write> mesh_input_buffer: array<MeshInput>;
@group(0) @binding(1) var<storage, read_write> mesh_culling_buffer: array<MeshCullingData>;
@group(0) @binding(2) var<storage, read> position: array<f32>;
#ifdef HAS_ROTATION
@group(0) @binding(3) var<storage, read> rotation: array<f32>;
#endif
#ifdef HAS_SCALE
@group(0) @binding(4) var<storage, read> scale: array<f32>;
#endif
#ifdef HAS_DEAD
@group(0) @binding(5) var<storage, read> dead: array<f32>;
#endif
@group(0) @binding(6) var<uniform> params: PackParams;

// Convert a unit quaternion (x, y, z, w) into a 3x3 rotation matrix expressed
// as three column vectors.
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
    let slot = params.base_input_index + i;

    let pos = vec3<f32>(
        position[i * 3u + 0u],
        position[i * 3u + 1u],
        position[i * 3u + 2u],
    );

#ifdef HAS_ROTATION
    let q = vec4<f32>(
        rotation[i * 4u + 0u],
        rotation[i * 4u + 1u],
        rotation[i * 4u + 2u],
        rotation[i * 4u + 3u],
    );
    let basis = quat_to_basis(q);
#else
    let basis = mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
    );
#endif

#ifdef HAS_SCALE
    let s = vec3<f32>(
        scale[i * 3u + 0u],
        scale[i * 3u + 1u],
        scale[i * 3u + 2u],
    );
#else
    let s = vec3<f32>(1.0, 1.0, 1.0);
#endif

    // mat3x4: 3 columns of vec4. Each column is one basis (x, y, z) row of the
    // affine, with the column's `w` storing the translation component.
    let c0 = basis[0] * s.x;
    let c1 = basis[1] * s.y;
    let c2 = basis[2] * s.z;
    mesh_input_buffer[slot].world_from_local = mat3x4<f32>(
        vec4<f32>(c0.x, c1.x, c2.x, pos.x),
        vec4<f32>(c0.y, c1.y, c2.y, pos.y),
        vec4<f32>(c0.z, c1.z, c2.z, pos.z),
    );
    mesh_input_buffer[slot].tag = i;

    mesh_culling_buffer[slot].aabb_center = vec3<f32>(0.0, 0.0, 0.0);
    mesh_culling_buffer[slot].aabb_half_extents = vec3<f32>(1.0, 1.0, 1.0);
#ifdef HAS_DEAD
    mesh_culling_buffer[slot].dead = dead[i];
#else
    mesh_culling_buffer[slot].dead = 0.0;
#endif
}
