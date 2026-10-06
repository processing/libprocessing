# Brightness Pixels
# by Daniel Shiffman.
#
# This program adjusts the brightness of a part of the image by
# calculating the distance of each pixel to the mouse.
from mewnala import *

img = None


def setup():
    global img
    size(640, 360)
    pixel_density(1)
    frame_rate(30)
    img = load_image("data/moon-wide.jpg")
    flush()  # make the image's pixels readable right away


def draw():
    px = img.pixels
    pixels = load_pixels()
    for x in range(img.width):
        for y in range(img.height):
            # Calculate the 1D location from a 2D grid
            loc = x + y * img.width
            # Get the R,G,B values from image
            r = px[loc].r
            # g = px[loc].g
            # b = px[loc].b
            # Calculate an amount to change brightness based on proximity to the mouse
            maxdist = 50  # dist(0, 0, width, height)
            d = dist(x, y, mouse_x, mouse_y)
            adjustbrightness = (maxdist - d) / maxdist
            r += adjustbrightness
            # g += adjustbrightness
            # b += adjustbrightness
            # Constrain RGB to make sure they are within 0-1 color range
            r = constrain(r, 0, 1)
            # g = constrain(g, 0, 1)
            # b = constrain(b, 0, 1)
            # Make a new color and set pixel in the window
            # c = color(r, g, b)
            c = color(r)
            pixels[y * width + x] = c
    update_pixels(pixels)


run()
