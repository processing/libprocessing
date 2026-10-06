# Sharpen.
#
# This program analyzes every pixel in an image and contrasts it with the
# neighboring pixels to sharpen the image.
#
# This is an example of an "image convolution" using a kernel (small matrix)
# to analyze and transform a pixel based on the values of its neighbors.
#
# Sharpening is also called a "high-pass filter".  Pixels of high frequency
# change (very differnt from neighbors) are left mostly unchanged, while
# those with low frequency change (similar value as neighbors) are modified
# greatly to increase contrast.
#
# The kernel here is a "high-boost filter", which is essentially the result
# of subtracting low-contrast (blurred) areas from the source image - leaving
# only the higher contrast sharp portions.
# A more advanced version is "unsharp masking", which allows greater control
# over the blur radius and sharpening amounts.
#
# For less severe sharpening, try this kernel:      [  0  -1   0 ]
#                                                   [ -1   5  -1 ]
#                                                   [  0  -1   0 ]
#
# For greater sharpening, try increasing the value of the center pixel.
from mewnala import *

kernel = [[-1, -1, -1],
          [-1, 9, -1],
          [-1, -1, -1]]

img = None
sharp_img = None


def setup():
    global img, sharp_img
    size(640, 360)
    img = load_image("data/moon.jpg")  # Load the original image
    # Create an opaque image of the same size as the original
    sharp_img = create_image(img.width, img.height)
    flush()  # make the images' pixels readable and writable right away
    no_loop()


def draw():
    image(img, 0, 0)  # Displays the image from point (0,0)
    px = img.pixels

    sharp_px = []
    for i in range(sharp_img.width * sharp_img.height):
        sharp_px.append(color(0.0))

    # Loop through every pixel in the image
    for y in range(1, img.height - 1):  # Skip top and bottom edges
        for x in range(1, img.width - 1):  # Skip left and right edges
            sum_red = 0  # Kernel sums for this pixel
            sum_green = 0
            sum_blue = 0
            for ky in range(-1, 2):
                for kx in range(-1, 2):
                    # Calculate the adjacent pixel for this kernel point
                    pos = (y + ky) * img.width + (x + kx)

                    # Process each channel separately, Red first.
                    val_red = px[pos].r
                    # Multiply adjacent pixels based on the kernel values
                    sum_red += kernel[ky + 1][kx + 1] * val_red

                    # Green
                    val_green = px[pos].g
                    sum_green += kernel[ky + 1][kx + 1] * val_green

                    # Blue
                    val_blue = px[pos].b
                    sum_blue += kernel[ky + 1][kx + 1] * val_blue
            # For this pixel in the new image, set the output value
            # based on the sum from the kernel
            sharp_px[y * sharp_img.width + x] = color(sum_red, sum_green, sum_blue)
    # State that there are changes to sharp_px
    sharp_img.update_pixels(sharp_px)

    image(sharp_img, width / 2, 0)  # Draw the new image


run()
