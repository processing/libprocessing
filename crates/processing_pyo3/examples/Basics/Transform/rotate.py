# Rotate.
#
# Rotating a square around the Z axis. To get the results
# you expect, send the rotate function angle parameters that are
# values between 0 and PI*2 (TWO_PI which is roughly 6.28). If you prefer to
# think about angles as degrees (0-360), you can use the radians()
# method to convert your values. For example: scale(radians(90))
# is identical to the statement scale(PI/2).
from mewnala import *
from datetime import datetime

angle = 0.0
jitter = 0.0


def setup():
    size(640, 360)
    no_stroke()
    fill(1.0)
    rect_mode(CENTER)


def draw():
    global angle, jitter
    background(0.2)

    # during even-numbered seconds (0, 2, 4, 6...)
    if datetime.now().second % 2 == 0:
        jitter = random(-0.1, 0.1)
    angle = angle + jitter
    c = cos(angle)
    translate(width / 2, height / 2)
    rotate(c)
    rect(0, 0, 180, 180)


run()
