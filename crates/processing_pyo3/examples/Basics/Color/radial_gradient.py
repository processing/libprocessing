# Radial Gradient.
#
# Draws a series of concentric circles to create a gradient
# from one color to another.
from mewnala import *

dim = 0


def setup():
    global dim
    size(640, 360)
    dim = width // 2
    background(0.0)
    no_stroke()
    ellipse_mode(RADIUS)
    frame_rate(1)


def draw():
    background(0.0)
    for x in range(0, width + 1, dim):
        draw_gradient(x, height / 2)


def draw_gradient(x, y):
    radius = dim // 2
    h = random(0, 360)
    for r in range(radius, 0, -1):
        fill(hsva(h, 0.9, 0.9))
        ellipse(x, y, r, r)
        h = (h + 1) % 360


run()
