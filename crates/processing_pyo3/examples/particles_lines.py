from mewnala import *
from math import cos, sin, tau

N = 3000
P, Q = 3, 2
SCALE = 8.0

p = None


def knot(t, phase):
    r = cos(Q * t) + 2.0
    x = r * cos(P * t + phase)
    y = r * sin(P * t + phase)
    z = -sin(Q * t)
    return (x * SCALE, y * SCALE, z * SCALE)


def setup():
    global p
    size(900, 700)
    window_title(f"Particle lines — {N:,}-point torus knot")
    mode_3d()

    p = create_particles(
        capacity=N,
        attributes=[Attribute.position()],
    )
    p.buffer("position").write([list(knot(i / N * tau, 0.0)) for i in range(N)])


def draw():
    background(6, 8, 14)

    phase = elapsed_time * 0.4
    positions = [list(knot(i / N * tau, phase)) for i in range(N)]
    p.buffer("position").write(positions)

    t = elapsed_time * 0.12
    r = SCALE * 4.5
    camera_position(cos(t) * r, SCALE * 1.5, sin(t) * r)
    camera_look_at(0.0, 0.0, 0.0)

    particles(p, topology="line_strip")


run()
