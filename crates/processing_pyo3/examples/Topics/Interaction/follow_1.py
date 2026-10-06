# Follow 1
# based on code from Keith Peters.
#
# A line segment is pushed and pulled by the cursor.
from mewnala import *

x = 100.0
y = 100.0
angle1 = 0.0
seg_length = 50


def setup():
    size(640, 360)
    stroke_weight(20.0)
    stroke(1.0, 0.39)


def draw():
    global x, y, angle1
    background(0.0)

    dx = mouse_x - x
    dy = mouse_y - y
    angle1 = atan2(dy, dx)
    x = mouse_x - (cos(angle1) * seg_length)
    y = mouse_y - (sin(angle1) * seg_length)

    segment(x, y, angle1)
    ellipse(x, y, 20, 20)


def segment(x, y, a):
    push_matrix()
    translate(x, y)
    rotate(a)
    line(0, 0, seg_length, 0)
    pop_matrix()


run()
