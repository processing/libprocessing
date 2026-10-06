# Map.
#
# Use the remap() function to take any number and scale it to a new number
# that is more useful for the project that you are working on. For example, use the
# numbers from the mouse position to control the size or color of a shape.
# In this example, the mouse's x-coordinate (numbers between 0 and 360) are scaled to
# new numbers to define the color and size of a circle.
from mewnala import *


def setup():
    size(640, 360)
    no_stroke()


def draw():
    background(0.0)
    # Scale the mouse_x value from 0 to 640 to a range between 0 and 175
    c = remap(mouse_x, 0, width, 0, 175)
    # Scale the mouse_x value from 0 to 640 to a range between 40 and 300
    d = remap(mouse_x, 0, width, 40, 300)
    fill(1.0, c / 255, 0.0)
    ellipse(width / 2, height / 2, d, d)


run()
