# Vector
# by Daniel Shiffman.
#
# Demonstration of some basic vector math: subtraction,
# normalization, scaling. Normalizing a vector sets
# its length to 1.
from mewnala import *


def setup():
    size(640, 360)


def draw():
    background(0.0)

    # A vector that points to the mouse location
    mouse = Vec2(mouse_x, mouse_y)

    # A vector that points to the center of the window
    center = Vec2(width / 2, height / 2)

    # Subtract center from mouse which results in a
    # vector that points from center to mouse
    mouse.sub(center)

    # Normalize the vector
    mouse = mouse.normalize()

    # Multiply its length by 150 (Scaling its length)
    mouse.mult(150)

    translate(width / 2, height / 2)
    # Draw the resulting vector
    stroke(1.0)
    stroke_weight(4)
    line(0, 0, mouse.x, mouse.y)


run()
