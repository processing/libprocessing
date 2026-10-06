# Move Eye.
# by Simon Greenwold.
#
# The camera lifts up (controlled by mouseY) while looking at the same point.
from mewnala import *

light = None


def setup():
    global light
    size(640, 360)
    mode_3d()
    fill(0.8)
    light = directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)


def draw():
    background(0.0)

    # Change height of the camera with mouseY (the world is y-up)
    camera(30.0, -mouse_y, 220.0,  # eyeX, eyeY, eyeZ
           0.0, 0.0, 0.0,  # centerX, centerY, centerZ
           0.0, 1.0, 0.0)  # upX, upY, upZ

    no_stroke()
    box(90)
    stroke(1.0)
    line(-100, 0, 0, 100, 0, 0)  # GAP: 3D line(x1, y1, z1, x2, y2, z2)
    line(0, -100, 0, 0, 100, 0)  # GAP: 3D line(x1, y1, z1, x2, y2, z2)
    line(0, 0, -100, 0, 0, 100)  # GAP: 3D line(x1, y1, z1, x2, y2, z2)


run()
