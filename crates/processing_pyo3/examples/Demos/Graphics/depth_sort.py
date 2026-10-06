# Depth sorting example by Jakub Valtar
# https://github.com/JakubValtar
#
# Processing sorts the translucent triangles back to front while the
# mouse is not pressed (hint(ENABLE_DEPTH_SORT)); hints have no equivalent here.
from mewnala import *

tris = []  # one triangle per hue


def setup():
    size(640, 720)
    mode_3d()
    frame_rate(60)
    # colorMode(HSB, 100, 100, 100, 100): the fill of each triangle becomes
    # its vertex color, built once per hue
    for i in range(10):
        c = hsva(remap(i, 0, 10, 0, 360), 1.0, 1.0, 0.3)
        tri = create_geometry(topology=TRIANGLES)
        tri.color(c.r, c.g, c.b, c.a)
        tri.vertex(200, 50, -50)
        tri.color(c.r, c.g, c.b, c.a)
        tri.vertex(100, 100, 50)
        tri.color(c.r, c.g, c.b, c.a)
        tri.vertex(100, 0, 20)
        tri.index(0)
        tri.index(1)
        tri.index(2)
        tris.append(tri)


def draw():
    no_stroke()

    background(0.0)

    translate(0, 0, -300)
    scale(2)

    rot = frame_count

    rotate_z(radians(90))
    rotate_x(radians(rot / 60.0 * 10))
    rotate_y(radians(rot / 60.0 * 30))

    blend_mode(ADD)

    for i in range(100):
        draw_geometry(tris[i % 10])
        rotate_y(radians(270.0 / 100))

    if frame_count % 30 == 0:
        print(frame_rate())


run()
