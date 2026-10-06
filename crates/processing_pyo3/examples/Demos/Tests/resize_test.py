# Resize Test
#
# The window can be resized by dragging its edges; the ellipse
# stays centered because width and height follow the window.
from mewnala import *


def setup():
    size(400, 400)
    mode_3d()
    window_resizable(True)


def draw():
    background(1.0, 0.0, 0.0)
    ellipse(0, 0, 100, 50)


run()
