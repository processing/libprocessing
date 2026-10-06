# Array 2D.
#
# Demonstrates the syntax for creating a two-dimensional (2D) array.
# Values in a 2D array are accessed through two index values.
# 2D arrays are useful for storing images. In this example, each dot
# is colored in relation to its distance from the center of the image.
from mewnala import *

distances = []
max_distance = 0.0
spacer = 0


def setup():
    global max_distance, spacer
    size(640, 360)
    max_distance = dist(width / 2, height / 2, width, height)
    for x in range(width):
        distances.append([0.0] * height)
    for y in range(height):
        for x in range(width):
            distance = dist(width / 2, height / 2, x, y)
            distances[x][y] = distance / max_distance
    spacer = 10
    stroke_weight(6)
    no_loop()  # Run once and stop


def draw():
    background(0.0)
    # This embedded loop skips over values in the arrays based on
    # the spacer variable, so there are more values in the array
    # than are drawn here. Change the value of the spacer variable
    # to change the density of the points
    for y in range(0, height, spacer):
        for x in range(0, width, spacer):
            stroke(distances[x][y])
            point(x + spacer // 2, y + spacer // 2)


run()
