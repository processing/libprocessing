# No Background Test
#
# The background is only drawn once in setup, so every frame
# accumulates on top of the previous one.
from mewnala import *


def setup():
    size(400, 400)
    background(1.0, 0.0, 0.0)
    fill(1.0, 0.59)


def draw():
    ellipse(mouse_x, mouse_y, 100, 100)


run()
