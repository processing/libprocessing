# Noise3D.
#
# Using 3D noise to create simple animated texture.
# Here, the third dimension ('z') is treated as time.
from mewnala import *

increment = 0.01
# The noise function's 3rd argument, a global variable that increments once per cycle
zoff = 0.0
# We will increment zoff differently than xoff and yoff
zincrement = 0.02


def setup():
    size(640, 360)
    pixel_density(1)
    frame_rate(30)


def draw():
    global zoff

    # Optional: adjust noise detail here
    # noise_detail(8, 0.65)

    px = load_pixels()

    xoff = 0.0  # Start xoff at 0

    # For every x,y coordinate in a 2D space, calculate a noise value and produce a brightness value
    for x in range(width):
        xoff += increment  # Increment xoff
        yoff = 0.0  # For every xoff, start yoff at 0
        for y in range(height):
            yoff += increment  # Increment yoff

            # Calculate noise, already in the 0..1 range
            bright = noise(xoff, yoff, zoff)

            # Try using this line instead
            # bright = random(1.0)

            # Set each pixel onscreen to a grayscale value
            px[x + y * width] = color(bright, bright, bright)
    update_pixels(px)

    zoff += zincrement  # Increment zoff


run()
