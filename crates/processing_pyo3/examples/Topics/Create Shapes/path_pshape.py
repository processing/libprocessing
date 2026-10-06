# PathPShape
#
# A simple path using PShape
from mewnala import *

# A PShape object
path = None


def setup():
    global path
    size(640, 360)

    # Create the shape
    path = create_shape()  # GAP: create_shape() retained shape with its own style
    path.begin_shape()  # GAP: Shape.begin_shape
    # Set fill and stroke
    path.no_fill()  # GAP: Shape.no_fill
    path.stroke(1.0)  # GAP: Shape.stroke
    path.stroke_weight(2)  # GAP: Shape.stroke_weight

    x = 0
    # Calculate the path as a sine wave
    a = 0.0
    while a < TWO_PI:
        path.vertex(x, sin(a) * 100)  # GAP: Shape.vertex
        x += 5
        a += 0.1
    # The path is complete
    path.end_shape()  # GAP: Shape.end_shape (open path)


def draw():
    background(0.2)
    # Draw the path at the mouse location
    translate(mouse_x, mouse_y)
    shape(path)  # GAP: shape(s) draws a retained shape


run()
