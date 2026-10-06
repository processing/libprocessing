# Create Image.
#
# The create_image() function provides a fresh buffer of pixels to play with.
# This example creates an image gradient.
from mewnala import *

img = None


def setup():
    global img
    size(640, 360)
    img = create_image(230, 230)
    count = img.width * img.height
    px = []
    for i in range(count):
        a = remap(i, 0, count, 1.0, 0.0)
        px.append(color(0.0, 0.6, 0.8, a))
    img.update_pixels(px)


def draw():
    background(0.0)
    image(img, 90, 80)
    image(img, mouse_x - img.width / 2, mouse_y - img.height / 2)


run()
