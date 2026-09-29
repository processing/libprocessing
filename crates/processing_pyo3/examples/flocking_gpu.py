# GPU flocking: the boids from flocking.py, built from generic particle ops
# over each boid's neighbors. Nothing is read back to the CPU.
from mewnala import *
from math import cos, sin
from random import uniform

BOID_COUNT = 100000
BOUND = 30.0  # half-extent of the wrapping box
NEIGHBOR_DIST = 5.0
SEPARATION_DIST = 2.5
MAX_SPEED = 10.0  # units per second
MAX_FORCE = 6.0  # units per second²
DT = 1.0 / 60.0

# Separation steers away from the summed offsets, so its speed is negative.
RULES = [
    ("separation", -MAX_SPEED, 1.5, "close"),
    ("alignment", MAX_SPEED, 1.0, "near"),
    ("cohesion", MAX_SPEED, 1.0, "near"),
]

p = None
boid = None
mat = None
grid = None
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
    global p, boid, mat, grid

    size(900, 700)
    window_title(f"GPU Flocking — {BOID_COUNT:,} boids")
    mode_3d()

    directional_light((0.95, 0.9, 0.85), 800.0)

    p = create_particles(BOID_COUNT)

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

    cells = int((2.0 * BOUND) / NEIGHBOR_DIST) + 1
    grid = p.create_grid(
        min=[-BOUND, -BOUND, -BOUND],
        cell_size=NEIGHBOR_DIST,
        dims=[cells, cells, cells],
    )


# Reynolds steering. `mask` zeroes it for boids with no neighbors in range,
# which would otherwise brake.
def steer(desired, speed, mask):
    p.apply(MAP, desired, op=NORMALIZE, length=speed)
    p.apply(COMBINE, desired, "velocity", op=SUB)
    p.apply(MAP, desired, op=LIMIT, max_length=MAX_FORCE * DT)
    p.apply(COMBINE, desired, mask, op=MUL)


def flock():
    p.apply(FIND_NEIGHBORS, grid=grid, radius=NEIGHBOR_DIST)
    # what each rule steers toward, summed over the neighbors
    p.apply(NEIGHBOR, "position", out="separation", op=SUM,
            relative=True, radius=SEPARATION_DIST, falloff=INVERSE_SQUARE)
    p.apply(NEIGHBOR, "velocity", out="alignment", op=SUM)
    p.apply(NEIGHBOR, "position", out="cohesion", op=SUM, relative=True)
    # which boids have any neighbors for each rule
    p.apply(NEIGHBOR, out="close", op=COUNT, radius=SEPARATION_DIST)
    p.apply(NEIGHBOR, out="near", op=COUNT)
    p.apply(MAP, "close", op=GREATER, threshold=0)
    p.apply(MAP, "near", op=GREATER, threshold=0)

    for i, (rule, speed, weight, mask) in enumerate(RULES):
        steer(rule, speed, mask)
        if i == 0:
            p.apply(MAP, rule, out="force", op=AFFINE, scale=weight)
        else:
            p.apply(COMBINE, "force", rule, op="add", b_scale=weight)
    p.apply(COMBINE, "velocity", "force", op="add")
    p.apply(MAP, "velocity", op=LIMIT, min_length=MAX_SPEED * 0.25, max_length=MAX_SPEED)


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

    flock()

    material(mat)
    particles(p, boid)

    p.apply(INTEGRATE, dt=DT)
    p.apply(BOUNDS_BOX, aabb_min=[-BOUND] * 3, aabb_max=[BOUND] * 3, mode=2)
    p.apply(ORIENT, forward=[0.0, 0.0, 1.0], up=[0.0, 1.0, 0.0])


run()
