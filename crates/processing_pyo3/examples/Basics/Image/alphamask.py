# Alpha Mask.
#
# Loads a "mask" for an image to specify the transparency
# in different parts of the image. The two images are blended
# together using the mask() method of an offscreen graphics buffer.
from mewnala import *

img = None
img_mask = None
masked = None


def setup():
    global img, img_mask, masked
    size(640, 360)
    img = load_image("data/moonwalk.jpg")
    img_mask = load_image("data/mask.jpg")
    # Apply the mask by drawing the image into a buffer and masking the buffer
    masked = create_graphics(img.width, img.height)
    masked.begin_draw()
    masked.image(img, 0, 0)
    masked.mask(img_mask)
    masked.end_draw()
    image_mode(CENTER)


def draw():
    background(0.0, 0.4, 0.6)
    image(masked, width / 2, height / 2)
    image(masked, mouse_x, mouse_y)


run()
