# Edge Detection
#
# Change the default shader to apply a simple, custom edge detection filter.
#
# Press the mouse to switch between the custom and default shader.
from mewnala import *

edges = None
img = None
enabled = True


def setup():
    global edges, img
    size(640, 360)
    img = load_image("data/leaves.jpg")
    edges = load_shader("data/edges.wesl")


def draw():
    image(img, 0, 0)
    if enabled:
        filter(edges)


def mouse_pressed():
    global enabled
    enabled = not enabled


run()
