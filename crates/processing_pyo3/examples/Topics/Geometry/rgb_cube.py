# RGB Cube.
#
# The three primary colors of the additive color model are red, green, and blue.
# This RGB color cube displays smooth transitions between these colors.
from mewnala import *

xmag = 0
ymag = 0
new_xmag = 0
new_ymag = 0
cube = None


def setup():
    global cube
    size(640, 360)
    mode_3d()
    no_stroke()
    cube = build_cube()


def draw():
    global xmag, ymag, new_xmag, new_ymag
    background(0.5)

    push_matrix()
    translate(0, 0, -30)

    new_xmag = mouse_x / width * TWO_PI
    new_ymag = mouse_y / height * TWO_PI

    diff = xmag - new_xmag
    if abs(diff) > 0.01:
        xmag -= diff / 4.0

    diff = ymag - new_ymag
    if abs(diff) > 0.01:
        ymag -= diff / 4.0

    rotate_x(-ymag)
    rotate_y(-xmag)

    scale(90)
    draw_geometry(cube)

    pop_matrix()


# Each corner's color is its position: (x, y, z) -> (r, g, b, 1)
def build_cube():
    g = create_geometry(topology=TRIANGLES)

    g.color(0, 1, 1, 1); g.vertex(-1, 1, 1)
    g.color(1, 1, 1, 1); g.vertex(1, 1, 1)
    g.color(1, 0, 1, 1); g.vertex(1, -1, 1)
    g.color(0, 0, 1, 1); g.vertex(-1, -1, 1)

    g.color(1, 1, 1, 1); g.vertex(1, 1, 1)
    g.color(1, 1, 0, 1); g.vertex(1, 1, -1)
    g.color(1, 0, 0, 1); g.vertex(1, -1, -1)
    g.color(1, 0, 1, 1); g.vertex(1, -1, 1)

    g.color(1, 1, 0, 1); g.vertex(1, 1, -1)
    g.color(0, 1, 0, 1); g.vertex(-1, 1, -1)
    g.color(0, 0, 0, 1); g.vertex(-1, -1, -1)
    g.color(1, 0, 0, 1); g.vertex(1, -1, -1)

    g.color(0, 1, 0, 1); g.vertex(-1, 1, -1)
    g.color(0, 1, 1, 1); g.vertex(-1, 1, 1)
    g.color(0, 0, 1, 1); g.vertex(-1, -1, 1)
    g.color(0, 0, 0, 1); g.vertex(-1, -1, -1)

    g.color(0, 1, 0, 1); g.vertex(-1, 1, -1)
    g.color(1, 1, 0, 1); g.vertex(1, 1, -1)
    g.color(1, 1, 1, 1); g.vertex(1, 1, 1)
    g.color(0, 1, 1, 1); g.vertex(-1, 1, 1)

    g.color(0, 0, 0, 1); g.vertex(-1, -1, -1)
    g.color(1, 0, 0, 1); g.vertex(1, -1, -1)
    g.color(1, 0, 1, 1); g.vertex(1, -1, 1)
    g.color(0, 0, 1, 1); g.vertex(-1, -1, 1)

    # Each group of four vertices is a quad: two triangles
    for face in range(6):
        for i in (0, 1, 2, 0, 2, 3):
            g.index(face * 4 + i)
    return g


run()
