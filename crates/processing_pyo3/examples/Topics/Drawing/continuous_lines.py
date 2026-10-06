# Continuous Lines.
#
# Click and drag the mouse to draw a line.
from mewnala import *


def setup():
    size(640, 360)
    background(0.4)


def draw():
    stroke(1.0)
    if mouse_is_pressed:
        line(mouse_x, mouse_y, pmouse_x, pmouse_y)


run()
