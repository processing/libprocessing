# Linear Image.
#
# Click and drag mouse up and down to control the signal.
# Press and hold any key to watch the scanning.
from mewnala import *

img = None
direction = 1

signal = 0.0


def setup():
    global img
    size(640, 360)
    pixel_density(1)
    stroke(1.0)
    img = load_image("data/sea.jpg")
    flush()  # make the image's pixels readable right away


def draw():
    global direction, signal
    if signal > img.height - 1 or signal < 0:
        direction = direction * -1
    if mouse_is_pressed:
        signal = abs(mouse_y % img.height)
    else:
        signal += 0.3 * direction

    if key_is_pressed:
        image(img, 0, 0)
        line(0, signal, img.width, signal)
    else:
        px = img.pixels
        pixels = load_pixels()
        signal_offset = int(signal) * img.width
        for y in range(img.height):
            pixels[y * width:y * width + img.width] = px[signal_offset:signal_offset + img.width]
        update_pixels(pixels)


run()
