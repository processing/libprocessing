# Edge Detection.
#
# This program analyzes every pixel in an image and compares it with thee
# neighboring pixels to identify edges.
#
# This is an example of an "image convolution" using a kernel (small matrix)
# to analyze and transform a pixel based on the values of its neighbors.
#
# This kernel describes a "Laplacian Edge Detector".  It is effective,
# but sensitive to noise.  One common enhancement is to add a Gaussian
# blur to the source image first, as in
#   gray_img.filter(BLUR)
# to reduce impact of noise on the output.  The combination is often called
# "Laplace of Gaussian", or "LoG" for short.
#
# For weaker detection effect, try this kernel:     [  0  -1   0 ]
#                                                   [ -1   4  -1 ]
#                                                   [  0  -1   0 ]
from mewnala import *

kernel = [[-1, -1, -1],
          [-1, 8, -1],
          [-1, -1, -1]]

img = None
edge_img = None


def setup():
    global img, edge_img
    size(640, 360)
    img = load_image("data/moon.jpg")  # Load the original image
    # Create an opaque image of the same size as the original
    edge_img = create_image(img.width, img.height)
    flush()  # make the images' pixels readable and writable right away
    no_loop()


def draw():
    image(img, 0, 0)  # Displays the image from point (0,0)

    # Edge detection should be done on a grayscale image.
    #  Create a copy of the source image, and convert to gray.
    gray_img = img.copy()  # GAP: Image.copy() (PImage.copy / p5 img.get())
    gray_img.filter(GRAY)  # GAP: Image.filter(GRAY) (PImage.filter / p5 img.filter)
    # gray_img.filter(BLUR)
    gray_px = gray_img.pixels

    edge_px = []
    for i in range(edge_img.width * edge_img.height):
        edge_px.append(color(0.0))

    # Loop through every pixel in the image
    for y in range(1, gray_img.height - 1):  # Skip top and bottom edges
        for x in range(1, gray_img.width - 1):  # Skip left and right edges
            # Output of this filter is shown as offset from 50% gray.
            # This preserves transitions from low (dark) to high (light) value.
            # Starting from zero will show only high edges on black instead.
            sum = 0.5
            for ky in range(-1, 2):
                for kx in range(-1, 2):
                    # Calculate the adjacent pixel for this kernel point
                    pos = (y + ky) * gray_img.width + (x + kx)

                    # Image is grayscale, red/green/blue are identical
                    val = gray_px[pos].b
                    # Multiply adjacent pixels based on the kernel values
                    sum += kernel[ky + 1][kx + 1] * val
            # For this pixel in the new image, set the output value
            # based on the sum from the kernel
            edge_px[y * edge_img.width + x] = color(sum)
    # State that there are changes to edge_px
    edge_img.update_pixels(edge_px)

    image(edge_img, width / 2, 0)  # Draw the new image


run()
