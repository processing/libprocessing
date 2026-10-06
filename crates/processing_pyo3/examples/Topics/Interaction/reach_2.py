# Reach 2
# based on code from Keith Peters.
#
# The arm follows the position of the mouse by
# calculating the angles with atan2().
from mewnala import *

num_segments = 10
x = [0.0] * num_segments
y = [0.0] * num_segments
angle = [0.0] * num_segments
seg_length = 26
target_x = 0.0
target_y = 0.0


def setup():
    size(640, 360)
    stroke_weight(20.0)
    stroke(1.0, 0.39)
    x[len(x) - 1] = width / 2  # Set base x-coordinate
    y[len(x) - 1] = height  # Set base y-coordinate


def draw():
    background(0.0)

    reach_segment(0, mouse_x, mouse_y)
    for i in range(1, num_segments):
        reach_segment(i, target_x, target_y)
    for i in range(len(x) - 1, 0, -1):
        position_segment(i, i - 1)
    for i in range(len(x)):
        segment(x[i], y[i], angle[i], (i + 1) * 2)


def position_segment(a, b):
    x[b] = x[a] + cos(angle[a]) * seg_length
    y[b] = y[a] + sin(angle[a]) * seg_length


def reach_segment(i, xin, yin):
    global target_x, target_y
    dx = xin - x[i]
    dy = yin - y[i]
    angle[i] = atan2(dy, dx)
    target_x = xin - cos(angle[i]) * seg_length
    target_y = yin - sin(angle[i]) * seg_length


def segment(x, y, a, sw):
    stroke_weight(sw)
    push_matrix()
    translate(x, y)
    rotate(a)
    line(0, 0, seg_length, 0)
    pop_matrix()


run()
