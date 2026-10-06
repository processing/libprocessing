# Text Rendering
#
# Draws the word HELLO 10000 times per frame at random positions.
from mewnala import *


def setup():
    size(800, 600)
    fill(0.0)


def draw():
    background(1.0)
    for i in range(10000):
        x = random(width)
        y = random(height)
        text("HELLO", x, y)
    if frame_count % 10 == 0:
        print(frame_rate())


run()
