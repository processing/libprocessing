# Shape Primitives.
#
# The basic shape primitive functions are triangle(),
# rect(), quad(), ellipse(), and arc(). Squares are made
# with rect() and circles are made with ellipse(). Each
# of these functions requires a number of parameters to
# determine the shape's position and size.
from mewnala import *


def setup():
    size(640, 360)
    no_stroke()
    no_loop()


def draw():
    background(0.0)

    fill(0.8)
    triangle(18, 18, 18, 360, 81, 360)

    fill(0.4)
    rect(81, 81, 63, 63)

    fill(0.8)
    quad(189, 18, 216, 18, 216, 360, 144, 360)

    fill(1.0)
    ellipse(252, 144, 72, 72)

    fill(0.8)
    triangle(288, 18, 351, 360, 288, 360)

    fill(1.0)
    arc(479, 300, 280, 280, PI, TWO_PI)


run()
