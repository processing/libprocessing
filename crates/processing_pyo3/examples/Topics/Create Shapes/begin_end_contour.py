# BeginEndContour
#
# How to cut a shape out of another using begin_contour() and end_contour()
from mewnala import *

angle = 0.0


def setup():
    size(640, 360)


def draw():
    global angle
    background(0.2)
    translate(width / 2, height / 2)
    # Shapes can be rotated
    angle += 0.01
    rotate(angle)

    # Make a shape
    fill(0.0)
    stroke(1.0)
    stroke_weight(2)
    begin_shape()
    # Exterior part of shape
    vertex(-100, -100)
    vertex(100, -100)
    vertex(100, 100)
    vertex(-100, 100)

    # Interior part of shape
    begin_contour()
    vertex(-10, -10)
    vertex(-10, 10)
    vertex(10, 10)
    vertex(10, -10)
    end_contour()

    # Finishing off shape
    end_shape(CLOSE)


run()
