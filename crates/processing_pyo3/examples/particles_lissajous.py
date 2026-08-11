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

N =  10000 # points on the curve
SCALE = 10.0
FX, FY, FZ = 3.0, 4.0, 5.0

CONNECTION_RADIUS = 3.5  # link points closer than this
CONNECTION_RAMP = 7.0    # alpha = (1/(d/R + 1))^ramp
LINE_ALPHA = 0.2        # overall opacity scale
MAX_LINKS = 4000           # per-point edge cap (bounds the edge buffer)
HUE_MIX = 0.0            # 0 = grayscale, 1 = rainbow

p = None       # curve points (source)
edges = None   # emitted line vertices (drawn)
curve = None
link = None
grid = None
idx = None


def setup():
    global p, edges, curve, link, grid, idx
    size(1000, 800)
    window_title(f"Lissajous — all points connected — {N:,} pts")
    mode_3d()
    bloom(0.0)

    p = create_particles(capacity=N, attributes=[Attribute.position(), Attribute.color()])
    # One line = 2 vertices; up to N*MAX_LINKS lines.
    edges = create_particles(
        capacity=N * MAX_LINKS * 2, attributes=[Attribute.position(), Attribute.color()]
    )
    idx = edges.index_buffer(N * MAX_LINKS * 2)  # link fills indices + the dynamic count

    cells = int((2.0 * SCALE + 2.0) / CONNECTION_RADIUS) + 1
    grid = p.create_grid(
        min=[-SCALE - 1.0] * 3, cell_size=CONNECTION_RADIUS, dims=[cells, cells, cells]
    )

    curve = create_compute(load_shader("shaders/plexus_curve.wesl"))
    link = create_compute(load_shader("shaders/plexus_link.wesl"))


def draw():
    bloom(0.0)  # re-assert each frame
    background(255, 255, 255)

    t = elapsed_time
    r = 26.0
    camera_position(cos(t * 0.08) * r, sin(t * 0.05) * r * 0.55, sin(t * 0.08) * r)
    camera_look_at(0.0, 0.0, 0.0)

    curve.set(count=N, time=t, fx=FX, fy=FY, fz=FZ, loops=1.0, scale=SCALE, hue_mix=HUE_MIX)
    p.apply(curve)

    grid.build(p.buffer("position"))
    edges.reset_indices()
    link.set(
        edge_pos=edges.buffer("position"),
        edge_col=edges.buffer("color"),
        indices=idx,
        draw_args=edges.draw_args(),
        connection_radius=CONNECTION_RADIUS,
        connection_ramp=CONNECTION_RAMP,
        line_alpha=LINE_ALPHA,
        max_links=MAX_LINKS,
    )
    grid.bind(link)
    p.apply(link)

    blend_mode(ALPHA_OVER)
    particles(edges, topology="lines")


run()
