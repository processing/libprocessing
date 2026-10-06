# Noise1D.
#
# Using 1D Perlin Noise to assign location.
from mewnala import *

xoff = 0.0
xincrement = 0.01


def setup():
    size(640, 360)
    background(0.0)
    no_stroke()


def draw():
    global xoff
    # Create an alpha blended background
    fill(0.0, 0.04)
    rect(0, 0, width, height)

    # n = random(0, width)  # Try this line instead of noise

    # Get a noise value based on xoff and scale it according to the window's width
    n = noise(xoff) * width

    # With each cycle, increment xoff
    xoff += xincrement

    # Draw the ellipse at the value produced by perlin noise
    fill(0.78)
    ellipse(n, height / 2, 64, 64)


run()
