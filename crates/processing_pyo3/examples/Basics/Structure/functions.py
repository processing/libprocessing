# Functions.
#
# The draw_target() function makes it easy to draw many distinct targets.
# Each call to draw_target() specifies the position, size, and number of
# rings for each target.
from mewnala import *


def setup():
    size(640, 360)
    background(0.2)
    no_stroke()
    no_loop()


def draw():
    draw_target(width * 0.25, height * 0.4, 200, 4)
    draw_target(width * 0.5, height * 0.5, 300, 10)
    draw_target(width * 0.75, height * 0.3, 120, 6)


def draw_target(xloc, yloc, size, num):
    grayvalues = 1.0 / num
    steps = size / num
    for i in range(num):
        fill(i * grayvalues)
        ellipse(xloc, yloc, size - i * steps, size - i * steps)


run()
