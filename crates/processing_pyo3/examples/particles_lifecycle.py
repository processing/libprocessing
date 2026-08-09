from mewnala import *
import math

p = None
sphere = None
mat = None
aging = None
frame = 0

BURST = 6
DT = 1.0 / 60.0
TTL = 1.0


def setup():
    global p, sphere, mat, aging

    size(900, 700)
    mode_3d()

    sphere = Geometry.sphere(0.1, 8, 6)

    capacity = 800
    p = create_particles(
        capacity=capacity,
        attributes=[
            Attribute.position(),
            Attribute.color(),
            Attribute.scale(),
            Attribute.life(),
            Attribute.age(),
        ],
    )
    color_buf = p.buffer("color")
    mat = create_material(unlit=True, albedo=color_buf)
    aging = create_compute(load_shader("shaders/particles_lifecycle_aging.wesl"))


def draw():
    global frame
    camera_position(0.0, 2.0, 14.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(10, 10, 18)

    material(mat)
    particles(p, sphere)

    positions = []
    colors = []
    for k in range(BURST):
        i = frame * BURST + k
        u = (((i * 2654435761) >> 8) & 0xFFFF) / 65535.0
        v = (((i * 40503) >> 8) & 0xFFFF) / 65535.0
        theta = u * math.tau
        r = v * 0.6
        positions.extend([math.cos(theta) * r, 2.5, math.sin(theta) * r])
        c = hsva((i * 4.68) % 360.0, 0.85, 1.0)
        colors.extend([c.r, c.g, c.b, 1.0])

    zeros = [0.0] * BURST
    ones = [1.0] * BURST
    ones_scale = [1.0] * (BURST * 3)
    p.emit(
        BURST,
        position=positions,
        color=colors,
        scale=ones_scale,
        age=zeros,
        life=ones,
    )

    aging.set(params=[DT, TTL, 0.0, 0.0])
    p.apply(aging)

    frame += 1


run()
