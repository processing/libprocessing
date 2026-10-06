# PrimitivePShape
#
# Using a PShape to display a primitive shape (in this case, ellipse).
from mewnala import *

# The PShape object
circle = None


def setup():
    global circle
    size(640, 360)
    # Creating the PShape as an ellipse
    circle = create_shape(ELLIPSE, 0, 0, 100, 50)  # GAP: create_shape(ELLIPSE, x, y, w, h) primitive shape


def draw():
    background(0.2)
    # We can dynamically set the stroke and fill of the shape
    circle.set_stroke(color(1.0))  # GAP: Shape.set_stroke(color)
    circle.set_stroke_weight(4)  # GAP: Shape.set_stroke_weight
    circle.set_fill(color(remap(mouse_x, 0, width, 0, 1)))  # GAP: Shape.set_fill(color)
    # We can use translate to move the PShape
    translate(mouse_x, mouse_y)
    # Drawing the PShape
    shape(circle)  # GAP: shape(s) draws a retained shape


run()
