from processing_py import *
from random import uniform
from math import sin

N = 300000
SEARCH = 0.5       # neighbour search radius

# the volume the particles live in; its walls are all outside the frame
HALF_X = 12.0
HALF_Y = 19.0
HALF_Z = 8.0
STEPS = 4          # small moves per frame

p = None
g = None
tint = None
contour = None


def setup():
    global p, g, tint, contour
    size(540, 960)
    window_title(f"Neighbour density — {N:,}")
    mode_3d()

    p = create_particles(
        N,
        attributes=[
            Attribute.position(),
            Attribute.color(),
            Attribute("density", AttributeFormat.Float),
        ],
    )

    positions = []
    for _ in range(N):
        x = uniform(-HALF_X, HALF_X)
        y = uniform(-HALF_Y, HALF_Y)
        z = uniform(-HALF_Z, HALF_Z)
        positions.append([x, y, z])
    p.buffer("position").write(positions)

    cells_x = int(2.0 * HALF_X / SEARCH) + 2
    cells_y = int(2.0 * HALF_Y / SEARCH) + 2
    cells_z = int(2.0 * HALF_Z / SEARCH) + 2
    g = p.create_grid(
        min=[-HALF_X, -HALF_Y, -HALF_Z],
        cell_size=SEARCH,
        dims=[cells_x, cells_y, cells_z],
    )

    tint = create_compute(load_shader("shaders/density_color.wesl"))

    contour = load_shader("shaders/density_contour.wesl")


def draw():
    background(3, 4, 9)

    t = elapsed_time
    camera_position(sin(t * 0.2) * 3.0, 0.0, 30.0)
    camera_look_at(0.0, 0.0, 0.0)

    # several small moves rather than one big one, wrapping at the walls after each
    for _ in range(STEPS):
        p.apply(NOISE, scale=0.12, strength=0.06 / STEPS, time=t * 0.15, divergence_free=1)
        p.apply(
            BOUNDS_BOX,
            aabb_min=[-HALF_X, -HALF_Y, -HALF_Z],
            aabb_max=[HALF_X, HALF_Y, HALF_Z],
            mode=2,
        )

    p.apply(NEIGHBOR, grid=g, out="density", op=DENSITY, radius=SEARCH)
    tint.set(scale=1.0 / 30.0)
    p.apply(tint)

    particles(p)

    # outline the gold: smear it, then draw a line where the smear crosses a level
    filter(contour, passes=3, ink=[0.62, 0.80, 1.0], spread=0.008, level=0.12, weight=1.5)


run()
