# GPU flocking: the boids from flocking.py, moved entirely onto the GPU.
# Positions and velocities live in particle attribute buffers, two compute
# kernels update them each frame, and the flock renders instanced — nothing
# is ever read back to the CPU. Brute-force O(N²) neighbor search is trivial
# for a GPU at this scale; a spatial hash grid is the next step past ~100k.
from mewnala import *
from math import cos, sin
from random import uniform

BOID_COUNT = 10000
BOUND = 30.0  # half-extent of the wrapping box
NEIGHBOR_DIST = 5.0
SEPARATION_DIST = 2.5
MAX_SPEED = 10.0  # units per second
MAX_FORCE = 6.0  # units per second²
DT = 1.0 / 60.0

# Pass 1: every boid reads the whole flock's state and writes only its
# steering force. Splitting the read from the write mirrors the CPU
# example's two loops — no boid sees a half-updated neighbor.

# Pass 2: integrate the steering force, wrap at the box edges, and point
# each instanced boid along its velocity via the rotation quaternion.

p = None
boid = None
mat = None
flock_pass = None
integrate_pass = None
title_last_time = 0.0
title_last_frame = 0


# Two triangles folded slightly along the nose-tail spine, like a paper
# boid pointing down +Z. The fold keeps the boid visible edge-on and gives
# each wing its own normal, so the flock glints as it banks.
def boid_geometry(half_width, length, droop):
    g = create_geometry()
    n = (half_width * half_width + droop * droop) ** 0.5
    nose = (0.0, 0.0, length * 0.5)
    tail = (0.0, 0.0, -length * 0.5)
    g.normal(-droop / n, half_width / n, 0.0)
    g.vertex(*nose)
    g.vertex(-half_width, -droop, -length * 0.5)
    g.vertex(*tail)
    g.normal(droop / n, half_width / n, 0.0)
    g.vertex(*nose)
    g.vertex(*tail)
    g.vertex(half_width, -droop, -length * 0.5)
    for i in range(6):
        g.index(i)
    return g


def setup():
    global p, boid, mat, flock_pass, integrate_pass

    size(900, 700)
    window_title(f"GPU Flocking — {BOID_COUNT:,} boids")
    mode_3d()

    directional_light((0.95, 0.9, 0.85), 800.0)

    p = create_particles(
        capacity=BOID_COUNT,
        attributes=[
            Attribute.position(),
            Attribute.rotation(),
            Attribute.color(),
            Attribute.velocity(),
            Attribute("steer", AttributeFormat.Float3),
        ],
    )

    positions = []
    velocities = []
    rotations = []
    colors = []
    for _ in range(BOID_COUNT):
        positions.append([uniform(-BOUND, BOUND) for _ in range(3)])
        velocities.append([uniform(-1.0, 1.0) * MAX_SPEED * 0.4 for _ in range(3)])
        rotations.append([0.0, 0.0, 0.0, 1.0])
        c = hsva(uniform(190.0, 280.0), 0.7, 1.0)
        colors.append([c.r, c.g, c.b, 1.0])

    p.buffer("position").write(positions)
    p.buffer("rotation").write(rotations)
    p.buffer("velocity").write(velocities)
    color_buf = p.buffer("color")
    color_buf.write(colors)

    boid = boid_geometry(0.4, 1.3, 0.15)
    mat = create_material(albedo=color_buf)

    flock_pass = create_compute(load_shader("shaders/flocking_gpu_flock.wesl"))
    integrate_pass = create_compute(load_shader("shaders/flocking_gpu_integrate.wesl"))


def draw():
    global title_last_time, title_last_frame

    title_elapsed = elapsed_time - title_last_time
    if title_elapsed >= 0.5:
        fps = (frame_count - title_last_frame) / title_elapsed
        window_title(f"GPU Flocking — {BOID_COUNT:,} boids — {fps:.0f} FPS")
        title_last_time = elapsed_time
        title_last_frame = frame_count

    t = elapsed_time * 0.1
    r = BOUND * 2.6
    camera_position(cos(t) * r, BOUND * 0.8, sin(t) * r)
    camera_look_at(0.0, 0.0, 0.0)
    background(10, 12, 18)

    material(mat)
    particles(p, boid)

    flock_pass.set(
        neighbor_dist=NEIGHBOR_DIST,
        separation_dist=SEPARATION_DIST,
        max_speed=MAX_SPEED,
        max_force=MAX_FORCE,
    )
    p.apply(flock_pass)

    integrate_pass.set(dt=DT, max_speed=MAX_SPEED, bound=BOUND)
    p.apply(integrate_pass)


run()
