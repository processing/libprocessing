# Texture Cube
# by Dave Bollinger.
#
# Drag mouse to rotate cube. Demonstrates use of u/v coords on a
# geometry and the effect of texture().
from mewnala import *

tex = None
cube = None
rotx = PI / 4
roty = PI / 4


def setup():
    global tex, cube
    size(640, 360)
    mode_3d()
    tex = load_image("data/berlin-1.jpg")
    cube = textured_cube()
    fill(1.0)
    stroke(0.17, 0.19, 0.13)


def draw():
    background(0.0)
    no_stroke()
    translate(0, 0, -100)
    rotate_x(rotx)
    rotate_y(roty)
    scale(90)
    unlit()
    texture(tex)
    draw_geometry(cube)
    no_texture()


def textured_cube():
    # Given one texture and six faces, we can easily set up the uv coordinates
    # such that four of the faces tile "perfectly" along either u or v, but the other
    # two faces cannot be so aligned.  This code tiles "along" u, "around" the X/Z faces
    # and fudges the Y faces - the Y faces are arbitrarily aligned such that a
    # rotation along the X axis will put the "top" of either texture at the "top"
    # of the screen, but is not otherwised aligned with the X/Z faces.
    faces = [
        # +Z "front" face
        [(-1, -1, 1, 0, 0), (1, -1, 1, 1, 0), (1, 1, 1, 1, 1), (-1, 1, 1, 0, 1)],
        # -Z "back" face
        [(1, -1, -1, 0, 0), (-1, -1, -1, 1, 0), (-1, 1, -1, 1, 1), (1, 1, -1, 0, 1)],
        # +Y "bottom" face
        [(-1, 1, 1, 0, 0), (1, 1, 1, 1, 0), (1, 1, -1, 1, 1), (-1, 1, -1, 0, 1)],
        # -Y "top" face
        [(-1, -1, -1, 0, 0), (1, -1, -1, 1, 0), (1, -1, 1, 1, 1), (-1, -1, 1, 0, 1)],
        # +X "right" face
        [(1, -1, 1, 0, 0), (1, -1, -1, 1, 0), (1, 1, -1, 1, 1), (1, 1, 1, 0, 1)],
        # -X "left" face
        [(-1, -1, -1, 0, 0), (-1, -1, 1, 1, 0), (-1, 1, 1, 1, 1), (-1, 1, -1, 0, 1)],
    ]
    g = create_geometry(topology=TRIANGLES)
    index = 0
    for face in faces:
        for (x, y, z, u, v) in face:
            # Attributes apply to the vertices added after them
            g.uv(u, v)
            g.vertex(x, y, z)
        # Each quad face is two triangles
        for i in (0, 1, 2, 0, 2, 3):
            g.index(index + i)
        index += 4
    return g


def mouse_dragged():
    global rotx, roty
    rate = 0.01
    rotx += (pmouse_y - mouse_y) * rate
    roty += (mouse_x - pmouse_x) * rate


run()
