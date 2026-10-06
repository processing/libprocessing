# Perspective vs. Ortho
#
# Move the mouse left to right to change the "far"
# parameter for the perspective() and ortho() functions.
# This parameter sets the maximum distance from the
# origin away from the viewer and will clip the geometry.
# Click a mouse button to switch between the perspective and
# orthographic projections.
from mewnala import *

show_perspective = False
light = None


def setup():
    global light
    size(600, 360)
    mode_3d()
    no_fill()
    fill(1.0)
    no_stroke()
    light = directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)


def draw():
    background(0.0)
    far = remap(mouse_x, 0, width, 120, 400)
    if show_perspective:
        perspective(PI / 3.0, width / height, 10, far)
    else:
        ortho(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0, 10, far)
    # Rotations about x run the other way in the y-up world
    rotate_x(PI / 6)
    rotate_y(PI / 3)
    box(180)


def mouse_pressed():
    global show_perspective
    show_perspective = not show_perspective


run()
