# Double Random
# by Ira Greenberg.
#
# Using two random() calls and the point() function
# to create an irregular sawtooth line.
from mewnala import *

total_pts = 300
steps = total_pts + 1


def setup():
    size(640, 360)
    stroke(1.0)
    frame_rate(1)


def draw():
    background(0.0)
    rand = 0
    for i in range(1, steps):
        point((width / steps) * i, (height / 2) + random(-rand, rand))
        rand += random(-5, 5)


run()
