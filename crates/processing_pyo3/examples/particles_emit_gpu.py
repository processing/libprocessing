from mewnala import *
import math

p = None
particle = None
mat = None
spawn = None
motion = None

CAPACITY = 40000
BURST = 120
DT = 1.0 / 60.0
TTL = 2.5
GRAVITY = 9.8
SPEED = 5.0

SPAWN_SHADER = """
struct Spawn {
    pos: vec4<f32>,
    speed: vec4<f32>,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> color: array<f32>;
@group(0) @binding(3) var<storage, read_write> scale: array<f32>;
@group(0) @binding(4) var<storage, read_write> age: array<f32>;
@group(0) @binding(5) var<storage, read_write> dead: array<f32>;
@group(0) @binding(6) var<uniform> spawn: Spawn;
@group(0) @binding(7) var<uniform> emit_range: vec4<f32>;

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
    if local_i >= u32(emit_range.y) { return; }
    let base = u32(emit_range.x);
    let cap  = u32(emit_range.z);
    let slot = (base + local_i) % cap;

    let seed = base + local_i;

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
    dead[slot] = 0.0;
}
"""

MOTION_SHADER = """
struct Params {
    dt: f32,
    ttl: f32,
    gravity: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> scale: array<f32>;
@group(0) @binding(3) var<storage, read_write> age: array<f32>;
@group(0) @binding(4) var<storage, read_write> dead: array<f32>;
@group(0) @binding(5) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count { return; }
    if dead[i] != 0.0 { return; }

    age[i] = age[i] + params.dt;

    velocity[i * 3u + 1u] = velocity[i * 3u + 1u] - params.gravity * params.dt;

    position[i * 3u + 0u] = position[i * 3u + 0u] + velocity[i * 3u + 0u] * params.dt;
    position[i * 3u + 1u] = position[i * 3u + 1u] + velocity[i * 3u + 1u] * params.dt;
    position[i * 3u + 2u] = position[i * 3u + 2u] + velocity[i * 3u + 2u] * params.dt;

    let life = clamp(1.0 - age[i] / params.ttl, 0.0, 1.0);
    let s = life * life;
    scale[i * 3u + 0u] = s;
    scale[i * 3u + 1u] = s;
    scale[i * 3u + 2u] = s;

    if age[i] > params.ttl { dead[i] = 1.0; }
}
"""


def setup():
    global p, particle, mat, spawn, motion

    size(900, 700)
    mode_3d()

    directional_light((0.95, 0.9, 0.85), 800.0)

    particle = Geometry.sphere(0.12, 8, 6)

    velocity_attr = Attribute("velocity", AttributeFormat.Float3)
    age_attr = Attribute("age", AttributeFormat.Float)

    p = Particles(
        capacity=CAPACITY,
        attributes=[
            Attribute.position(),
            Attribute.color(),
            Attribute.scale(),
            Attribute.dead(),
            velocity_attr,
            age_attr,
        ],
    )

    # Mark all unemitted slots dead so they don't render at origin.
    dead_buf = p.buffer(Attribute.dead())
    dead_buf.write([1.0] * CAPACITY)

    color_buf = p.buffer(Attribute.color())
    mat = Material.pbr(albedo=color_buf)

    spawn = Compute(Shader(SPAWN_SHADER))
    motion = Compute(Shader(MOTION_SHADER))


def draw():
    camera_position(0.0, 4.0, 16.0)
    camera_look_at(0.0, 2.0, 0.0)
    background(10, 10, 18)

    use_material(mat)
    particles(p, particle)

    # Animate spawn point in a small circle so the fountain meanders.
    t = elapsed_time
    sx = math.cos(t) * 0.4
    sz = math.sin(t) * 0.4
    spawn.set(pos=[sx, 7.0, sz, 0.0], speed=[SPEED, 0.0, 0.0, 0.0])
    p.emit_gpu(BURST, spawn)

    motion.set(dt=DT, ttl=TTL, gravity=GRAVITY)
    p.apply(motion)


run()
