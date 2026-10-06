# Noise Wave
# by Daniel Shiffman.
#
# Using Perlin Noise to generate a wave-like pattern.
from mewnala import *

yoff = 0.0  # 2nd dimension of perlin noise


def setup():
    size(640, 360)


def draw():
    global yoff
    background(0.2)

    fill(1.0)
    # We are going to draw a polygon out of the wave points
    begin_shape()

    xoff = 0  # Option #1: 2D Noise
    # xoff = yoff  # Option #2: 1D Noise

    # Iterate over horizontal pixels
    for x in range(0, width + 1, 10):
        # Calculate a y value according to noise, map to
        y = remap(noise(xoff, yoff), 0, 1, 200, 300)  # Option #1: 2D Noise
        # y = remap(noise(xoff), 0, 1, 200, 300)  # Option #2: 1D Noise

        # Set the vertex
        vertex(x, y)
        # Increment x dimension for noise
        xoff += 0.05
    # increment y dimension for noise
    yoff += 0.01
    vertex(width, height)
    vertex(0, height)
    end_shape(CLOSE)


run()
