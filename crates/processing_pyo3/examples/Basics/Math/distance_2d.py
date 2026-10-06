# Distance 2D.
#
# Move the mouse across the image to obscure and reveal the matrix.
# Measures the distance from the mouse to each square and sets the
# size proportionally.
from mewnala import *

max_distance = 0.0


def setup():
    global max_distance
    size(640, 360)
    no_stroke()
    max_distance = dist(0, 0, width, height)


def draw():
    background(0.0)

    for i in range(0, width + 1, 20):
        for j in range(0, height + 1, 20):
            size = dist(mouse_x, mouse_y, i, j)
            size = size / max_distance * 66
            ellipse(i, j, size, size)


run()
