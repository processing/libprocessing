from mewnala import *
from math import cos, sin
from random import uniform

COUNT = 20000
BOUND = 20.0

p = None


def setup():
    global p
    size(900, 700)
    window_title(f"Particle points — {COUNT:,}")
    mode_3d()

    p = create_particles(
        capacity=COUNT,
        attributes=[Attribute.position()],
    )

    positions = [[uniform(-BOUND, BOUND) for _ in range(3)] for _ in range(COUNT)]
    p.buffer("position").write(positions)


def draw():
    background(6, 8, 14)

    t = elapsed_time * 0.15
    r = BOUND * 2.6
    camera_position(cos(t) * r, BOUND * 0.6, sin(t) * r)
    camera_look_at(0.0, 0.0, 0.0)

    particles(p)


run()
