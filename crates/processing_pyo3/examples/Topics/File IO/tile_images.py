# Tile Images
#
# Draws an image larger than the screen, and saves the image as nine tiles.
# The scale_value variable sets amount of scaling: 1 is 100%, 2 is 200%, etc.
from mewnala import *

scale_value = 3  # Multiplication factor
xoffset = 0  # x-axis offset
yoffset = 0  # y-axis offset


def setup():
    size(600, 600)
    stroke(0.0, 0.39)


def draw():
    background(0.8)
    scale(scale_value)
    translate(xoffset * (-width / scale_value), yoffset * (-height / scale_value))
    line(10, 150, 500, 50)
    line(0, 600, 600, 0)
    save("lines-" + str(yoffset) + "-" + str(xoffset) + ".png")
    set_offset()


def set_offset():
    global xoffset, yoffset
    xoffset += 1
    if xoffset == scale_value:
        xoffset = 0
        yoffset += 1
        if yoffset == scale_value:
            print("Tiles saved.")
            exit_sketch()


run()
