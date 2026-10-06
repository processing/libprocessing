# Width and Height.
#
# The 'width' and 'height' variables contain the width and height
# of the display window as defined in the size() function.
from mewnala import *


def setup():
    size(640, 360)


def draw():
    background(0.5)
    no_stroke()
    for i in range(0, height, 20):
        fill(0.51, 0.81, 0.06)
        rect(0, i, width, 10)
        fill(1.0)
        rect(i, 0, 10, height)


run()
