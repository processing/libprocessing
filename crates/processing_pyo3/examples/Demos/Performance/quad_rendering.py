# Quad Rendering
#
# Draws 50000 nearly transparent squares per frame.
from mewnala import *


def setup():
    size(800, 600)

    no_stroke()
    fill(0.0, 0.004)


def draw():
    background(1.0)
    for i in range(50000):
        x = random(width)
        y = random(height)
        rect(x, y, 30, 30)
    if frame_count % 10 == 0:
        print(frame_rate())


run()
