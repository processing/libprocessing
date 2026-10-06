# Save One Image
#
# The save() function allows you to save an image from the
# display window. In this example, save() is run when a mouse
# button is pressed. The image "line.png" is saved to the
# same folder as the sketch's program file.
from mewnala import *


def setup():
    size(640, 360)


def draw():
    background(0.8)
    line(0, 0, mouse_x, height)
    line(width, 0, 0, mouse_y)


def mouse_pressed():
    save("line.png")


run()
