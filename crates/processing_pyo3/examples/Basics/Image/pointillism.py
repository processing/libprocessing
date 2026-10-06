# Pointillism
# by Daniel Shiffman.
#
# Mouse horizontal location controls size of dots.
# Creates a simple pointillist effect using ellipses colored
# according to pixels in an image.
from mewnala import *

img = None
small_point = 0
large_point = 0


def setup():
    global img, small_point, large_point
    size(640, 360)
    img = load_image("data/moonwalk.jpg")
    small_point = 4
    large_point = 40
    image_mode(CENTER)
    no_stroke()
    background(1.0)


def draw():
    pointillize = remap(mouse_x, 0, width, small_point, large_point)
    x = int(random(img.width))
    y = int(random(img.height))
    pix = img.get(x, y)
    fill(pix.with_alpha(0.5))
    ellipse(x, y, pointillize, pointillize)


run()
