# Get Child.
#
# SVG files can be made of many individual shapes.
# Each of these shapes (called a "child") has its own name
# that can be used to extract it from the "parent" file.
# This example loads a map of the United States and creates
# two new Shape objects by extracting the data from two states.
from mewnala import *

usa = None
michigan = None
ohio = None


def setup():
    global usa, michigan, ohio
    size(640, 360)
    usa = load_shape("data/usa-wikipedia.svg")  # GAP: load_shape() (SVG) is missing
    michigan = usa.get_child("MI")  # GAP: Shape.get_child(name) by SVG id
    ohio = usa.get_child("OH")


def draw():
    background(1.0)

    # Draw the full map
    shape(usa, -600, -180)  # GAP: shape(s, x, y)

    # Disable the colors found in the SVG file
    michigan.disable_style()  # GAP: Shape.disable_style()
    # Set our own coloring
    fill(0.0, 0.2, 0.4)
    no_stroke()
    # Draw a single state
    shape(michigan, -600, -180)  # Wolverines!

    # Disable the colors found in the SVG file
    ohio.disable_style()
    # Set our own coloring
    fill(0.6, 0.0, 0.0)
    no_stroke()
    # Draw a single state
    shape(ohio, -600, -180)  # Buckeyes!


run()
