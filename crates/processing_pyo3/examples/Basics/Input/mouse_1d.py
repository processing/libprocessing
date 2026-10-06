# Mouse 1D.
#
# Move the mouse left and right to shift the balance.
# The "mouse_x" variable is used to control both the
# size and color of the rectangles.
from mewnala import *


def setup():
    size(640, 360)
    no_stroke()
    rect_mode(CENTER)


def draw():
    background(0.0)

    r1 = remap(mouse_x, 0, width, 0, height)
    r2 = height - r1

    fill(r1 / height)
    rect(width / 2 + r1 / 2, height / 2, r1, r1)

    fill(r2 / height)
    rect(width / 2 - r2 / 2, height / 2, r2, r2)


run()
