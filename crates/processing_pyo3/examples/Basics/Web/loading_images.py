# Loading Images.
#
# Processing applications can load images from the network.
from mewnala import *

img = None


def setup():
    global img
    size(640, 360)
    img = load_image("https://processing.org/img/processing-web.png")  # GAP: load_image() from a URL
    no_loop()


def draw():
    background(0.0)
    if img is not None:
        for i in range(5):
            image(img, 0, img.height * i)


run()
