from mewnala import *
from random import uniform, seed
from math import cos, sin

N = 2500          # particles
BOX = 12.0        # half-extent of the box
LINK_DIST = 1.8
MAX_LINKS = 8     # per-particle edge cap
SPEED = 3.0       # units/sec
DT = 1.0 / 60.0

BOUNCE = 1        # bounds_box mode: 0=clamp, 1=reflect(bounce), 2=wrap

p = None
link = None
grid = None
idx = None
args = None


def setup():
    global p, link, grid, idx, args
    size(1000, 760)
    window_title(f"Plexus — {N:,} particles, dynamic GPU links")
    mode_3d()

    p = create_particles(
        capacity=N,
        attributes=[Attribute.position(), Attribute.velocity(), Attribute.color()],
    )

    seed(7)
    positions, velocities, colors = [], [], []
    for _ in range(N):
        positions.append([uniform(-BOX, BOX) for _ in range(3)])
        velocities.append([uniform(-1.0, 1.0) * SPEED for _ in range(3)])
        c = hsva(uniform(170.0, 320.0), 0.65, 1.0)
        colors.append([c.r, c.g, c.b, 1.0])
    p.buffer("position").write(positions)
    p.buffer("velocity").write(velocities)
    p.buffer("color").write(colors)

    # sized for the worst case: every particle at its cap (2 indices per edge)
    idx = p.index_buffer(N * MAX_LINKS * 2)
    args = p.draw_args()

    # cell_size = LINK_DIST so the 27-cell neighbour walk covers the search radius
    cells = int((2.0 * BOX) / LINK_DIST) + 1
    grid = p.create_grid(min=[-BOX, -BOX, -BOX], cell_size=LINK_DIST, dims=[cells, cells, cells])

    link = create_compute(load_shader("shaders/plexus_link.wesl"))


def draw():
    background(6, 8, 14)

    t = elapsed_time * 0.12
    r = BOX * 3.0
    camera_position(cos(t) * r, BOX * 0.9, sin(t) * r)
    camera_look_at(0.0, 0.0, 0.0)

    p.apply(INTEGRATE, dt=DT)
    p.apply(BOUNDS_BOX, aabb_min=[-BOX] * 3, aabb_max=[BOX] * 3, mode=BOUNCE, max_speed=SPEED)

    grid.build(p.buffer("position"))
    p.reset_indices()
    link.set(indices=idx, draw_args=args, link_distance=LINK_DIST, max_links=MAX_LINKS)
    grid.bind(link)
    p.apply(link)

    particles(p, topology="lines")


run()
