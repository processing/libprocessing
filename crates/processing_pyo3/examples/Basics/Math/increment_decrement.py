# Increment Decrement.
#
# Writing "a++" is equivalent to "a = a + 1".
# Writing "a--" is equivalent to "a = a - 1".
from mewnala import *

a = 0
b = 0
direction = True


def setup():
    global a, b, direction
    size(640, 360)
    a = 0
    b = width
    direction = True
    frame_rate(30)


def draw():
    global a, b, direction
    a += 1
    if a > width:
        a = 0
        direction = not direction
    if direction == True:
        stroke(a / width)
    else:
        stroke((width - a) / width)
    line(a, 0, a, height / 2)

    b -= 1
    if b < 0:
        b = width
    if direction == True:
        stroke((width - b) / width)
    else:
        stroke(b / width)
    line(b, height / 2 + 1, b, height)


run()
