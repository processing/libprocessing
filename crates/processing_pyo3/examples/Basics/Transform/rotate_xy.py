# Rotate 1.
#
# Rotating simultaneously in the X and Y axis.
# Transformation functions such as rotate() are additive.
# Successively calling rotate(1.0) and rotate(2.0)
# is equivalent to calling rotate(3.0).
from mewnala import *

a = 0.0
r_size = 0  # rectangle size


def setup():
    global r_size
    size(640, 360)
    mode_3d()
    r_size = width / 6
    no_stroke()
    fill(0.8, 0.8)


def draw():
    global a
    background(0.49)

    a += 0.005
    if a > TWO_PI:
        a = 0.0

    rotate_x(a)
    rotate_y(a * 2.0)
    fill(1.0)
    rect(-r_size, -r_size, r_size * 2, r_size * 2)

    rotate_x(a * 1.001)
    rotate_y(a * 2.002)
    fill(0.0)
    rect(-r_size, -r_size, r_size * 2, r_size * 2)


run()
