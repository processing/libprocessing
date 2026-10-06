# Line Rendering
#
# Draws 50000 translucent random lines per frame.
from mewnala import *


def setup():
    size(800, 600)


def draw():
    background(1.0)
    stroke(0.0, 0.04)
    for i in range(50000):
        x0 = random(width)
        y0 = random(height)
        x1 = random(width)
        y1 = random(height)

        # The original passes random z coordinates too (ignored by P2D);
        # line() is 2D here.
        line(x0, y0, x1, y1)
    if frame_count % 10 == 0:
        print(frame_rate())


run()
