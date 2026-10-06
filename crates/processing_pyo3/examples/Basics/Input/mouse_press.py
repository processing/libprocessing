# Mouse Press.
#
# Move the mouse to position the shape.
# Press the mouse button to invert the color.
from mewnala import *


def setup():
    size(640, 360)
    fill(0.49)
    background(0.4)


def draw():
    if mouse_is_pressed:
        stroke(1.0)
    else:
        stroke(0.0)
    line(mouse_x - 66, mouse_y, mouse_x + 66, mouse_y)
    line(mouse_x, mouse_y - 66, mouse_x, mouse_y + 66)


run()
