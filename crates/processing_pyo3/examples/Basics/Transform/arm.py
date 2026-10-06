# Arm.
#
# The angle of each segment is controlled with the mouse_x and
# mouse_y position. The transformations applied to the first segment
# are also applied to the second segment because they are inside
# the same push_matrix() and pop_matrix() group.
from mewnala import *

x = 0
y = 0
angle1 = 0.0
angle2 = 0.0
seg_length = 100


def setup():
    global x, y
    size(640, 360)
    stroke_weight(30)
    stroke(1.0, 0.63)

    x = width * 0.3
    y = height * 0.5


def draw():
    global angle1, angle2
    background(0.0)

    angle1 = (mouse_x / width - 0.5) * -PI
    angle2 = (mouse_y / height - 0.5) * PI

    push_matrix()
    segment(x, y, angle1)
    segment(seg_length, 0, angle2)
    pop_matrix()


def segment(x, y, a):
    translate(x, y)
    rotate(a)
    line(0, 0, seg_length, 0)


run()
