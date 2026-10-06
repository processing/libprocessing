# Zoom.
#
# Move the cursor over the image to alter its position. Click and press
# the mouse to zoom. This program displays a series of lines with their
# heights corresponding to a color value read from an image.
from mewnala import *

img = None
img_pixels = []
sval = 1.0
nmx = 0.0
nmy = 0.0
res = 5


def setup():
    global img
    size(640, 360)
    mode_3d()
    no_fill()
    stroke(1.0)
    img = load_image("data/ystone08.jpg")
    flush()  # make the image's pixels readable right away
    for j in range(img.width):
        img_pixels.append([0] * img.height)
    for i in range(img.height):
        for j in range(img.width):
            img_pixels[j][i] = img.get(j, i)


def draw():
    global sval, nmx, nmy
    background(0.0)

    nmx += (mouse_x - nmx) / 20
    nmy += (mouse_y - nmy) / 20

    if mouse_is_pressed:
        sval += 0.005
    else:
        sval -= 0.01

    sval = constrain(sval, 1.0, 2.0)

    translate(nmx * sval - 100, -(nmy * sval - 100), -50)
    scale(sval)
    rotate_z(PI / 9 - sval + 1.0)
    rotate_x(PI / sval / 8 - 0.125)
    rotate_y(sval / 8 - 0.125)

    translate(-width / 2, height / 2, 0)

    for i in range(0, img.height, res):
        for j in range(0, img.width, res):
            rr = img_pixels[j][i].r
            gg = img_pixels[j][i].g
            bb = img_pixels[j][i].b
            tt = (rr + gg + bb) * 255
            stroke(rr, gg, gg)
            line(i, -j, tt / 10 - 20, i, -j, tt / 10)  # GAP: 3D line(x1, y1, z1, x2, y2, z2)


run()
