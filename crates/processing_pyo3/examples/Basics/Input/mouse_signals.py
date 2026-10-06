# Mouse Signals
#
# Move and click the mouse to generate signals.
# The top row is the signal from "mouse_x",
# the middle row is the signal from "mouse_y",
# and the bottom row is the signal from "mouse_is_pressed".
from mewnala import *

xvals = []
yvals = []
bvals = []


def setup():
    global xvals, yvals, bvals
    size(640, 360)
    xvals = [0] * width
    yvals = [0] * width
    bvals = [0] * width


def draw():
    background(0.4)

    for i in range(1, width):
        xvals[i - 1] = xvals[i]
        yvals[i - 1] = yvals[i]
        bvals[i - 1] = bvals[i]
    # Add the new values to the end of the array
    xvals[width - 1] = mouse_x
    yvals[width - 1] = mouse_y

    if mouse_is_pressed:
        bvals[width - 1] = 0
    else:
        bvals[width - 1] = height // 3

    fill(1.0)
    no_stroke()
    rect(0, height // 3, width, height // 3 + 1)

    for i in range(1, width):
        # Draw the x-values
        stroke(1.0)
        point(i, remap(xvals[i], 0, width, 0, height // 3 - 1))

        # Draw the y-values
        stroke(0.0)
        point(i, height // 3 + yvals[i] // 3)

        # Draw the mouse presses
        stroke(1.0)
        line(i, (2 * height // 3) + bvals[i], i, (2 * height // 3) + bvals[i - 1])


run()
