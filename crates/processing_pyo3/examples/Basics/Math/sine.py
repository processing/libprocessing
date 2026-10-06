# Sine.
#
# Smoothly scaling size with the sin() function.
from mewnala import *

diameter = 0.0
angle = 0


def setup():
    global diameter
    size(640, 360)
    diameter = height - 10
    no_stroke()
    fill(1.0, 0.8, 0.0)


def draw():
    global angle

    background(0.0)

    d1 = 10 + (sin(angle) * diameter / 2) + diameter / 2
    d2 = 10 + (sin(angle + PI / 2) * diameter / 2) + diameter / 2
    d3 = 10 + (sin(angle + PI) * diameter / 2) + diameter / 2

    ellipse(0, height / 2, d1, d1)
    ellipse(width / 2, height / 2, d2, d2)
    ellipse(width, height / 2, d3, d3)

    angle += 0.02


run()
