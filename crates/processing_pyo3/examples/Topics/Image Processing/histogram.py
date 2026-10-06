# Histogram.
#
# Calculates the histogram of an image.
# A histogram is the frequency distribution
# of the gray levels with the number of pure black values
# displayed on the left and number of pure white values on the right.
from mewnala import *

img = None


def setup():
    global img
    size(640, 360)
    # Load an image from the data directory
    # Load a different image by modifying the comments
    img = load_image("data/frontier.jpg")
    flush()  # make the image's pixels readable right away
    no_loop()


def draw():
    image(img, 0, 0)
    hist = [0] * 256

    # Calculate the histogram
    px = img.pixels
    for i in range(img.width):
        for j in range(img.height):
            bright = int(brightness(px[i + j * img.width]) * 255)
            hist[bright] += 1

    # Find the largest value in the histogram
    hist_max = max(hist)

    stroke(1.0)
    # Draw half of the histogram (skip every second value)
    for i in range(0, img.width, 2):
        # Map i (from 0..img.width) to a location in the histogram (0..255)
        which = int(remap(i, 0, img.width, 0, 255))
        # Convert the histogram value to a location between
        # the bottom and the top of the picture
        y = int(remap(hist[which], 0, hist_max, img.height, 0))
        line(i, img.height, i, y)


run()
