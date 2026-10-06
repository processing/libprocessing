# Transparency.
#
# Move the pointer left and right across the image to change
# its position. This program overlays one image over another
# by modifying the alpha value of the image with the tint() function.
from mewnala import *

img = None
offset = 0.0
easing = 0.05


def setup():
    global img
    size(640, 360)
    img = load_image("data/moonwalk.jpg")  # Load an image into the program


def draw():
    global offset
    image(img, 0, 0)  # Display at full opacity
    dx = (mouse_x - img.width / 2) - offset
    offset += dx * easing
    tint(1.0, 0.5)  # Display at half opacity
    image(img, offset, 0)


run()
