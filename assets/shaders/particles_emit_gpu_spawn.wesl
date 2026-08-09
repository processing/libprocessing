struct Spawn {
    pos: vec4<f32>,
    speed: vec4<f32>,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> color: array<f32>;
@group(0) @binding(3) var<storage, read_write> scale: array<f32>;
@group(0) @binding(4) var<storage, read_write> age: array<f32>;
@group(0) @binding(5) var<storage, read_write> life: array<f32>;
@group(0) @binding(6) var<uniform> spawn: Spawn;
@group(0) @binding(7) var<uniform> emit_base: u32;
@group(0) @binding(8) var<uniform> emit_count: u32;
@group(0) @binding(9) var<uniform> emit_capacity: u32;

fn hash(n: u32) -> u32 {
    var x = n;
    x = (x ^ 61u) ^ (x >> 16u);
    x = x + (x << 3u);
    x = x ^ (x >> 4u);
    x = x * 0x27d4eb2du;
    x = x ^ (x >> 15u);
    return x;
}

fn hash_unit(n: u32) -> f32 {
    return f32(hash(n)) / f32(0xffffffffu);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let local_i = gid.x;
    if local_i >= emit_count { return; }
    let slot = (emit_base + local_i) % emit_capacity;

    let seed = emit_base + local_i;

    let theta = hash_unit(seed) * 6.2831853;
    let r     = sqrt(hash_unit(seed * 2u + 1u));
    let dirxz = vec2<f32>(cos(theta), sin(theta)) * r;
    let dy    = 0.7 + 0.3 * hash_unit(seed * 3u + 7u);
    let v     = vec3<f32>(dirxz.x, dy, dirxz.y) * spawn.speed.x;

    position[slot * 3u + 0u] = spawn.pos.x;
    position[slot * 3u + 1u] = spawn.pos.y;
    position[slot * 3u + 2u] = spawn.pos.z;

    velocity[slot * 3u + 0u] = v.x;
    velocity[slot * 3u + 1u] = v.y;
    velocity[slot * 3u + 2u] = v.z;

    let h = fract(hash_unit(seed * 5u + 11u));
    color[slot * 4u + 0u] = 0.5 + 0.5 * sin(h * 6.28);
    color[slot * 4u + 1u] = 0.5 + 0.5 * sin(h * 6.28 + 2.094);
    color[slot * 4u + 2u] = 0.5 + 0.5 * sin(h * 6.28 + 4.189);
    color[slot * 4u + 3u] = 1.0;

    scale[slot * 3u + 0u] = 1.0;
    scale[slot * 3u + 1u] = 1.0;
    scale[slot * 3u + 2u] = 1.0;

    age[slot]  = 0.0;
    life[slot] = 1.0;
}
