# Blur.
#
# This program analyzes every pixel in an image and blends it with the
# neighboring pixels to blur the image.
#
# This is an example of an "image convolution" using a kernel (small matrix)
# to analyze and transform a pixel based on the values of its neighbors.
#
# Image blur is also called a "low-pass filter".  Pixels of low frequency
# change (similar brightness as neighbors) are left mostly unchanged, while
# those with high frequency change (sharply different values) are smoothed
# out.
#
# The kernel here is a Box Blur, in which all components are equally valued.
# Another common blur is "Gaussian Blur", in which pixels nearer the center
# of the kernel have more weight than those further away.
#
# An example 3x3 Gaussian kernel might be:          [ 1  2  1 ]
#                                            1/16 * [ 2  4  2 ]
#                                                   [ 1  2  1 ]
#
# An example 5x5 kernel, which creates a greater blur effect:
#                                                   [ 1   4   6   4  1 ]
#                                                   [ 4  16  24  16  4 ]
#                                           1/256 * [ 6  24  36  24  6 ]
#                                                   [ 4  16  24  16  4 ]
#                                                   [ 1   4   6   4  1 ]
from mewnala import *

v = 1.0 / 9.0
kernel = [[v, v, v],
          [v, v, v],
          [v, v, v]]

img = None
blur_img = None


def setup():
    global img, blur_img
    size(640, 360)
    img = load_image("data/moon.jpg")  # Load the original image
    # Create an opaque image of the same size as the original
    blur_img = create_image(img.width, img.height)
    flush()  # make the images' pixels readable and writable right away
    no_loop()


def draw():
    image(img, 0, 0)  # Displays the image from point (0,0)
    px = img.pixels

    blur_px = []
    for i in range(blur_img.width * blur_img.height):
        blur_px.append(color(0.0))

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
            blur_px[y * blur_img.width + x] = color(sum_red, sum_green, sum_blue)
    # State that there are changes to blur_px
    blur_img.update_pixels(blur_px)

    image(blur_img, width / 2, 0)  # Draw the new image


run()
