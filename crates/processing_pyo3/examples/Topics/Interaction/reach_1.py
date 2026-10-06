# Reach 1
# based on code from Keith Peters.
#
# The arm follows the position of the mouse by
# calculating the angles with atan2().
from mewnala import *

seg_length = 80
x = 0.0
y = 0.0
x2 = 0.0
y2 = 0.0


def setup():
    global x, y, x2, y2
    size(640, 360)
    stroke_weight(20.0)
    stroke(1.0, 0.39)

    x = width / 2
    y = height / 2
    x2 = x
    y2 = y


def draw():
    global x, y
    background(0.0)

    dx = mouse_x - x
    dy = mouse_y - y
    angle1 = atan2(dy, dx)

    tx = mouse_x - cos(angle1) * seg_length
    ty = mouse_y - sin(angle1) * seg_length
    dx = tx - x2
    dy = ty - y2
    angle2 = atan2(dy, dx)
    x = x2 + cos(angle2) * seg_length
    y = y2 + sin(angle2) * seg_length

    segment(x, y, angle1)
    segment(x2, y2, angle2)


def segment(x, y, a):
    push_matrix()
    translate(x, y)
    rotate(a)
    line(0, 0, seg_length, 0)
    pop_matrix()


run()
