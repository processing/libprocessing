# Follow 3
# based on code from Keith Peters.
#
# A segmented line follows the mouse. The relative angle from
# each segment to the next is calculated with atan2() and the
# position of the next is calculated with sin() and cos().
from mewnala import *

x = [0.0] * 20
y = [0.0] * 20
seg_length = 18


def setup():
    size(640, 360)
    stroke_weight(9)
    stroke(1.0, 0.39)


def draw():
    background(0.0)
    drag_segment(0, mouse_x, mouse_y)
    for i in range(len(x) - 1):
        drag_segment(i + 1, x[i], y[i])


def drag_segment(i, xin, yin):
    dx = xin - x[i]
    dy = yin - y[i]
    angle = atan2(dy, dx)
    x[i] = xin - cos(angle) * seg_length
    y[i] = yin - sin(angle) * seg_length
    segment(x[i], y[i], angle)


def segment(x, y, a):
    push_matrix()
    translate(x, y)
    rotate(a)
    line(0, 0, seg_length, 0)
    pop_matrix()


run()
