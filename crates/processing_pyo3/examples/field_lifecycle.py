from mewnala import *
import math

field_obj = None
sphere = None
mat = None
aging = None
position_attr = None
color_attr = None
scale_attr = None
dead_attr = None
age_attr = None
frame = 0

BURST = 6
DT = 1.0 / 60.0
TTL = 1.0

AGING_SHADER = """
@group(0) @binding(0) var<storage, read_write> age: array<f32>;
@group(0) @binding(1) var<storage, read_write> dead: array<f32>;
@group(0) @binding(2) var<storage, read_write> position: array<f32>;
@group(0) @binding(3) var<storage, read_write> scale: array<f32>;
@group(0) @binding(4) var<uniform> params: vec4<f32>;  // x = dt, y = ttl

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count {
        return;
    }
    let dt = params.x;
    let ttl = params.y;

    if dead[i] != 0.0 {
        return;
    }

    age[i] = age[i] + dt;
    position[i * 3u + 1u] = position[i * 3u + 1u] - dt * 1.5;

    let life = clamp(1.0 - age[i] / ttl, 0.0, 1.0);
    let s = life * life;
    scale[i * 3u + 0u] = s;
    scale[i * 3u + 1u] = s;
    scale[i * 3u + 2u] = s;

    if age[i] > ttl {
        dead[i] = 1.0;
    }
}
"""


def setup():
    global field_obj, sphere, mat, aging
    global position_attr, color_attr, scale_attr, dead_attr, age_attr

    size(900, 700)
    mode_3d()

    sphere = Geometry.sphere(0.1, 8, 6)

    capacity = 800
    position_attr = Attribute.position()
    color_attr = Attribute.color()
    scale_attr = Attribute.scale()
    dead_attr = Attribute.dead()
    age_attr = Attribute("age", AttributeFormat.Float)

    field_obj = Field(
        capacity=capacity,
        attributes=[position_attr, color_attr, scale_attr, dead_attr, age_attr],
    )

    # Mark all slots dead initially so unemitted ring slots don't render.
    dead_buf = field_obj.buffer(dead_attr)
    dead_buf.write([1.0] * capacity)

    color_buf = field_obj.buffer(color_attr)
    mat = Material.unlit(albedo=color_buf)
    aging = Compute(Shader(AGING_SHADER))


def draw():
    global frame
    camera_position(0.0, 2.0, 14.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(10, 10, 18)

    use_material(mat)
    draw_field(field_obj, sphere)

    # Spawn `BURST` new particles per frame in a small fountain.
    positions = []
    colors = []
    for k in range(BURST):
        i = frame * BURST + k
        # Cheap pseudo-random offset.
        u = (((i * 2654435761) >> 8) & 0xFFFF) / 65535.0
        v = (((i * 40503) >> 8) & 0xFFFF) / 65535.0
        theta = u * math.tau
        r = v * 0.6
        positions.extend([math.cos(theta) * r, 2.5, math.sin(theta) * r])
        c = hsva((i * 4.68) % 360.0, 0.85, 1.0)
        colors.extend([c.r, c.g, c.b, 1.0])

    zeros = [0.0] * BURST
    ones_scale = [1.0] * (BURST * 3)
    field_obj.emit(
        BURST,
        position=positions,
        color=colors,
        scale=ones_scale,
        age=zeros,
        dead=zeros,
    )

    aging.set(params=[DT, TTL, 0.0, 0.0])
    field_obj.apply(aging)

    frame += 1


run()
