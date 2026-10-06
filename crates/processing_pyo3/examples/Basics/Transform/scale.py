# Scale
# by Denis Grutze.
#
# Paramenters for the scale() function are values specified
# as decimal percentages. For example, the method call scale(2.0)
# will increase the dimension of the shape by 200 percent.
# Objects always scale from the origin.
from mewnala import *

a = 0.0
s = 0.0


def setup():
    size(640, 360)
    no_stroke()
    rect_mode(CENTER)
    frame_rate(30)


def draw():
    global a, s
    background(0.4)

    a = a + 0.04
    s = cos(a) * 2

    translate(width / 2, height / 2)
    scale(s)
    fill(0.2)
    rect(0, 0, 50, 50)

    translate(75, 0)
    fill(1.0)
    scale(s)
    rect(0, 0, 50, 50)


run()
