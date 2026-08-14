from mewnala import *
from math import cos, sin

ALPHA_OVER = BlendMode(
    color_src=BlendMode.SRC_ALPHA,
    color_dst=BlendMode.ONE_MINUS_SRC_ALPHA,
    color_op=BlendMode.OP_ADD,
    alpha_src=BlendMode.ONE,
    alpha_dst=BlendMode.ONE_MINUS_SRC_ALPHA,
    alpha_op=BlendMode.OP_ADD,
)

N = 10000
SCALE = 10.0
FX, FY, FZ = 3.0, 4.0, 5.0

CONNECTION_RADIUS = 3.5  # link points closer than this
CONNECTION_RAMP = 6.0    # alpha = (1/(d/R + 1))^ramp
LINE_ALPHA = 0.5         # overall opacity scale
MAX_LINES = 2_000_000    # capacity of the dynamic line set (extra dropped, see edges.overflowed())
HUE_MIX = 0.0            # 0 = grayscale, 1 = rainbow

p = None
edges = None
curve = None
link = None
grid = None


def setup():
    global p, edges, curve, link, grid
    size(1000, 800)
    window_title(f"Lissajous — all points connected — {N:,} pts")
    mode_3d()
    bloom(0.0)

    p = create_particles(capacity=N, attributes=[Attribute.position(), Attribute.color()])
    edges = p.primitives("lines", capacity=MAX_LINES)

    cells = int((2.0 * SCALE + 2.0) / CONNECTION_RADIUS) + 1
    grid = p.create_grid(
        min=[-SCALE - 1.0] * 3, cell_size=CONNECTION_RADIUS, dims=[cells, cells, cells]
    )
    curve = create_compute(load_shader("shaders/plexus_curve.wesl"))
    link = create_compute(load_shader("shaders/plexus_link.wesl"))
    orbit_camera()


def draw():
    bloom(0.0)  # re-assert each frame
    background(255, 255, 255)

    t = elapsed_time

    curve.set(count=N, time=t, fx=FX, fy=FY, fz=FZ, loops=1.0, scale=SCALE, hue_mix=HUE_MIX)
    p.apply(curve)

    grid.build(p.buffer("position"))
    link.set(
        connection_radius=CONNECTION_RADIUS,
        connection_ramp=CONNECTION_RAMP,
        line_alpha=LINE_ALPHA,
    )
    grid.bind(link)
    p.apply(link, primitives=edges)

    blend_mode(ALPHA_OVER)
    particles(edges)


run()
