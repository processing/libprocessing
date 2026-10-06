# Simple Linear Gradient
#
# The lerp_color() function is useful for interpolating
# between two colors.
from mewnala import *

# Constants
Y_AXIS = 1
X_AXIS = 2
b1 = None
b2 = None
c1 = None
c2 = None


def setup():
    global b1, b2, c1, c2
    size(640, 360)

    # Define colors
    b1 = color(1.0)
    b2 = color(0.0)
    c1 = color(0.8, 0.4, 0.0)
    c2 = color(0.0, 0.4, 0.6)

    no_loop()


def draw():
    # Background
    set_gradient(0, 0, width / 2, height, b1, b2, X_AXIS)
    set_gradient(width / 2, 0, width / 2, height, b2, b1, X_AXIS)
    # Foreground
    set_gradient(50, 90, 540, 80, c1, c2, Y_AXIS)
    set_gradient(50, 190, 540, 80, c2, c1, X_AXIS)


def set_gradient(x, y, w, h, c1, c2, axis):
    no_fill()

    if axis == Y_AXIS:  # Top to bottom gradient
        for i in range(int(y), int(y + h) + 1):
            inter = remap(i, y, y + h, 0, 1)
            c = lerp_color(c1, c2, inter)
            stroke(c)
            line(x, i, x + w, i)
    elif axis == X_AXIS:  # Left to right gradient
        for i in range(int(x), int(x + w) + 1):
            inter = remap(i, x, x + w, 0, 1)
            c = lerp_color(c1, c2, inter)
            stroke(c)
            line(i, y, i, y + h)


run()
