# Linear Motion.
#
# Changing a variable to create a moving line.
# When the line moves off the edge of the window,
# the variable is set to 0, which places the line
# back at the bottom of the screen.
from mewnala import *

a = 0.0


def setup():
    global a
    size(640, 360)
    stroke(1.0)
    a = height / 2


def draw():
    global a
    background(0.2)
    line(0, a, width, a)
    a = a - 0.5
    if a < 0:
        a = height


run()
