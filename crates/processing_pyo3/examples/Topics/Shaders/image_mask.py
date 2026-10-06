# Image Mask
#
# Move the mouse to reveal the image through the dynamic mask.
from mewnala import *

mask_shader = None
src_image = None
mask_image = None
masked = None


def setup():
    global mask_shader, src_image, mask_image, masked
    size(640, 360)
    src_image = load_image("data/leaves2.jpg")
    mask_image = create_graphics(src_image.width, src_image.height)
    # the mask shader runs as a filter on an offscreen copy of the image,
    # which is then drawn (with its transparency) over the canvas
    masked = create_graphics(src_image.width, src_image.height)
    mask_shader = load_shader("data/mask.wesl")
    background(1.0)


def draw():
    mask_image.begin_draw()
    mask_image.background(0.0)
    if mouse_x != 0 and mouse_y != 0:
        mask_image.no_stroke()
        mask_image.fill(1.0, 0.0, 0.0)
        mask_image.ellipse(mouse_x, mouse_y, 50, 50)
    mask_image.end_draw()

    masked.begin_draw()
    masked.image(src_image, 0, 0)
    masked.filter(mask_shader, mask=mask_image)
    masked.end_draw()
    image(masked, 0, 0, width, height)


run()
