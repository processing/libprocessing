# Pixel Array.
#
# Click and drag the mouse up and down to control the signal and
# press and hold any key to see the current pixel being read.
# This program sequentially reads the color of every pixel of an image
# and displays this color to fill the window.
from mewnala import *

img = None
direction = 1
signal = 0.0


def setup():
    global img
    size(640, 360)
    no_fill()
    stroke(1.0)
    frame_rate(30)
    img = load_image("data/sea.jpg")
    flush()  # make the image's pixels readable right away


def draw():
    global direction, signal
    if signal > img.width * img.height - 1 or signal < 0:
        direction = direction * -1

    if mouse_is_pressed:
        mx = constrain(mouse_x, 0, img.width - 1)
        my = constrain(mouse_y, 0, img.height - 1)
        signal = my * img.width + mx
    else:
        signal += 0.33 * direction

    sx = int(signal) % img.width
    sy = int(signal) // img.width

    if key_is_pressed:
        image(img, 0, 0)  # fast way to draw an image
        point(sx, sy)
        rect(sx - 5, sy - 5, 10, 10)
    else:
        c = img.get(sx, sy)
        background(c)


run()
