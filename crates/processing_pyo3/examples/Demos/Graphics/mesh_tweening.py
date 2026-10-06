# Mesh Tweening
#
# Use of custom vertex attributes.
# Inspired by
# http://pyopengl.sourceforge.net/context/tutorials/shader_4.html
#
# Every vertex carries a second position ("tweened") and the vertex shader
# blends between the two with the mouse.
from mewnala import *

sh = None
mat = None
grid = None


def setup():
    global sh, mat, grid
    size(640, 360)
    mode_3d()
    sh = load_shader("data/tween.wesl")  # GAP: GLSL vert/frag pairs don't load; custom materials are WESL (data/tween.wesl is the port)
    mat = create_material(sh, tween=0.0)

    grid = create_geometry(topology=TRIANGLES)
    d = 10
    for x in range(-500, 500, d):
        for y in range(-500, 500, d):
            base = grid.vertex_count()
            for (cx, cy) in [(x, y), (x + d, y), (x + d, y + d), (x, y + d)]:
                n = noise(cx, cy)
                grid.color(n, n, n, 1.0)
                grid.attribute("tweened", cx, cy, 100 * n)  # GAP: no custom per-vertex attributes on Geometry (PShape.attribPosition)
                grid.vertex(cx, cy, 0)
            for i in (0, 1, 2, 0, 2, 3):
                grid.index(base + i)


def draw():
    background(1.0)

    mat.set(tween=remap(mouse_x, 0, width, 0, 1))

    rotate_x(frame_count * 0.01)
    rotate_y(frame_count * 0.01)

    no_stroke()
    material(mat)
    draw_geometry(grid)


run()
