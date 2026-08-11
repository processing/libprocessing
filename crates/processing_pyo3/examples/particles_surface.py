from mewnala import *
from math import cos, sin, tau

SEGMENTS = 400
N = SEGMENTS * 2
R = 10.0
WIDTH = 3.5

p = None


def band(seg, phase):
    theta = seg / SEGMENTS * tau
    radial = (cos(theta), 0.0, sin(theta))
    center = (R * radial[0], 0.0, R * radial[2])
    twist = theta * 0.5 + phase
    w = (
        WIDTH * cos(twist) * radial[0],
        WIDTH * sin(twist),
        WIDTH * cos(twist) * radial[2],
    )
    left = [center[0] - w[0], center[1] - w[1], center[2] - w[2]]
    right = [center[0] + w[0], center[1] + w[1], center[2] + w[2]]
    return left, right


def setup():
    global p
    size(900, 700)
    window_title(f"Particle surface — {N:,}-vertex Mobius strip")
    mode_3d()

    p = create_particles(capacity=N, attributes=[Attribute.position()])


def draw():
    background(6, 8, 14)

    phase = elapsed_time * 0.25
    positions = []
    for seg in range(SEGMENTS):
        left, right = band(seg, phase)
        positions.append(left)
        positions.append(right)
    p.buffer("position").write(positions)

    t = elapsed_time * 0.15
    r = R * 3.0
    camera_position(cos(t) * r, R * 1.2, sin(t) * r)
    camera_look_at(0.0, 0.0, 0.0)

    particles(p, topology="triangle_strip")


run()
