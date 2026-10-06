# Sine Cosine.
#
# Linear movement with sin() and cos().
# Numbers between 0 and PI*2 (TWO_PI which angles roughly 6.28)
# are put into these functions and numbers between -1 and 1 are
# returned. These values are then scaled to produce larger movements.
from mewnala import *

x1 = 0.0
x2 = 0.0
y1 = 0.0
y2 = 0.0
angle1 = 0.0
angle2 = 0.0
scalar = 70


def setup():
    size(640, 360)
    no_stroke()
    rect_mode(CENTER)


def draw():
    global x1, x2, y1, y2, angle1, angle2
    background(0.0)

    ang1 = radians(angle1)
    ang2 = radians(angle2)

    x1 = width / 2 + (scalar * cos(ang1))
    x2 = width / 2 + (scalar * cos(ang2))

    y1 = height / 2 + (scalar * sin(ang1))
    y2 = height / 2 + (scalar * sin(ang2))

    fill(1.0)
    rect(width * 0.5, height * 0.5, 140, 140)

    fill(0.0, 0.4, 0.6)
    ellipse(x1, height * 0.5 - 120, scalar, scalar)
    ellipse(x2, height * 0.5 + 120, scalar, scalar)

    fill(1.0, 0.8, 0.0)
    ellipse(width * 0.5 - 120, y1, scalar, scalar)
    ellipse(width * 0.5 + 120, y2, scalar, scalar)

    angle1 += 2
    angle2 += 3


run()
