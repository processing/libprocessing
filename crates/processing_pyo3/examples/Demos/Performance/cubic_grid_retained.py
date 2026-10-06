# CubicGridRetained
#
# The same translucent grid as CubicGridImmediate, but all the boxes are
# built once into a single geometry and drawn with one call per frame.
from mewnala import *

box_size = 20
margin = box_size * 2
depth = 400
box_fill = None

grid = None

fcount = 0
lastm = 0
frate = 0
fint = 3


def setup():
    global grid, box_fill
    size(640, 360)
    mode_3d()
    frame_rate(60)
    no_stroke()

    grid = create_geometry(topology=TRIANGLES)

    # Build grid using multiple translations
    i = -depth / 2 + margin
    while i <= depth / 2 - margin:
        j = -height + margin
        while j <= height - margin:
            k = -width + margin
            while k <= width - margin:
                # Base fill color on counter values, abs function
                # ensures values stay within legal range
                box_fill = color(constrain(abs(i), 0, 255) / 255, constrain(abs(j), 0, 255) / 255, constrain(abs(k), 0, 255) / 255, 0.2)
                add_box(grid, k, j, i, box_size, box_fill)
                k += box_size
            j += box_size
        i += box_size


def draw():
    global fcount, lastm, frate
    background(1.0)

    # Center and spin grid
    push_matrix()
    translate(0, 0, -depth)
    rotate_y(frame_count * 0.01)
    rotate_x(frame_count * 0.01)

    draw_geometry(grid)
    pop_matrix()

    fcount += 1
    m = millis()
    if m - lastm > 1000 * fint:
        frate = fcount / fint
        fcount = 0
        lastm = m
        print("fps: " + str(frate))
    window_title("fps: " + str(frate))


# Appends a colored box (8 vertices, 12 triangles) centered at (x, y, z)
def add_box(g, x, y, z, s, c):
    h = s / 2
    base = g.vertex_count()
    for (dx, dy, dz) in [(-1, -1, -1), (1, -1, -1), (1, 1, -1), (-1, 1, -1), (-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1)]:
        g.color(c.r, c.g, c.b, c.a)
        g.vertex(x + dx * h, y + dy * h, z + dz * h)
    for (a, b, d) in [(0, 2, 1), (0, 3, 2), (4, 5, 6), (4, 6, 7), (0, 1, 5), (0, 5, 4), (2, 3, 7), (2, 7, 6), (1, 2, 6), (1, 6, 5), (0, 4, 7), (0, 7, 3)]:
        g.index(base + a)
        g.index(base + b)
        g.index(base + d)


run()
