# Background Image.
#
# This example presents the fastest way to load a background image
# into Processing. To load an image as the background, it must be
# the same width and height as the program.
from mewnala import *

bg = None
y = 0


def setup():
    global bg
    size(640, 360)
    # The background image must be the same size as the parameters
    # into the size() method. In this program, the size of the image
    # is 640 x 360 pixels.
    bg = load_image("data/moonwalk.jpg")


def draw():
    global y
    background(bg)

    stroke(0.89, 0.8, 0.0)
    line(0, y, width, y)

    y += 1
    if y > height:
        y = 0


run()
