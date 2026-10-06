# Color Variables (Homage to Albers).
#
# This example creates variables for colors that may be referred to
# in the program by a name, rather than a number.
from mewnala import *

inside = None
middle = None
outside = None


def setup():
    global inside, middle, outside
    size(640, 360)
    no_stroke()
    no_loop()

    inside = color(0.8, 0.4, 0.0)
    middle = color(0.8, 0.6, 0.0)
    outside = color(0.6, 0.2, 0.0)

    # These statements are equivalent to the statements above.
    # Programmers may use the format they prefer.
    # inside = color("#CC6600")
    # middle = color("#CC9900")
    # outside = color("#993300")


def draw():
    background(0.2, 0.0, 0.0)

    push_matrix()
    translate(80, 80)
    fill(outside)
    rect(0, 0, 200, 200)
    fill(middle)
    rect(40, 60, 120, 120)
    fill(inside)
    rect(60, 90, 80, 80)
    pop_matrix()

    push_matrix()
    translate(360, 80)
    fill(inside)
    rect(0, 0, 200, 200)
    fill(outside)
    rect(40, 60, 120, 120)
    fill(middle)
    rect(60, 90, 80, 80)
    pop_matrix()


run()
