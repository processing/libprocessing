# Distance 1D.
#
# Move the mouse left and right to control the
# speed and direction of the moving shapes.
from mewnala import *

xpos1 = 0.0
xpos2 = 0.0
xpos3 = 0.0
xpos4 = 0.0
thin = 8
thick = 36


def setup():
    global xpos1, xpos2, xpos3, xpos4
    size(640, 360)
    no_stroke()
    xpos1 = width / 2
    xpos2 = width / 2
    xpos3 = width / 2
    xpos4 = width / 2


def draw():
    global xpos1, xpos2, xpos3, xpos4
    background(0.0)

    mx = mouse_x * 0.4 - width / 5.0

    fill(0.4)
    rect(xpos2, 0, thick, height / 2)
    fill(0.8)
    rect(xpos1, 0, thin, height / 2)
    fill(0.4)
    rect(xpos4, height / 2, thick, height / 2)
    fill(0.8)
    rect(xpos3, height / 2, thin, height / 2)

    xpos1 += mx / 16
    xpos2 += mx / 64
    xpos3 -= mx / 16
    xpos4 -= mx / 64

    if xpos1 < -thin:
        xpos1 = width
    if xpos1 > width:
        xpos1 = -thin
    if xpos2 < -thick:
        xpos2 = width
    if xpos2 > width:
        xpos2 = -thick
    if xpos3 < -thin:
        xpos3 = width
    if xpos3 > width:
        xpos3 = -thin
    if xpos4 < -thick:
        xpos4 = width
    if xpos4 > width:
        xpos4 = -thick


run()
