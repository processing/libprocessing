# Convolution
# by Daniel Shiffman.
#
# Applies a convolution matrix to a portion of an image. Move mouse to
# apply filter to different parts of the image. Click mouse to cycle
# through different effects (kernels).
from mewnala import *

img = None
effect = 0
w = 120

# It's possible to convolve the image with many different
# matrices to produce different effects.  Here are some
# example kernels to try.
identity = [[0, 0, 0],
            [0, 1, 0],
            [0, 0, 0]]

darken = [[0, 0, 0],
          [0, 0.5, 0],
          [0, 0, 0]]

lighten = [[0, 0, 0],
           [0, 2, 0],
           [0, 0, 0]]

sharpen = [[0, -1, 0],
           [-1, 5, -1],
           [0, -1, 0]]

sharpen2 = [[-1, -1, -1],
            [-1, 9, -1],
            [-1, -1, -1]]

box_blur = [[1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0],
            [1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0],
            [1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0]]

edge_det = [[0, 1, 0],
            [1, -4, 1],
            [0, 1, 0]]

emboss = [[-2, -1, 0],
          [-1, 1, 1],
          [0, 1, 2]]

# collect the kernels and names into lists for our program
kernels = [
    identity,
    darken,
    lighten,
    sharpen,
    sharpen2,
    box_blur,
    edge_det,
    emboss,
]

effect_names = [
    "Identity (no change)",
    "Darken",
    "Lighten",
    "Sharpen",
    "Sharpen More",
    "Box Blur",
    "Edge Detect",
    "Emboss",
]


def setup():
    global img
    size(640, 360)
    pixel_density(1)
    img = load_image("data/moon-wide.jpg")
    flush()  # make the image's pixels readable right away

    no_loop()


# Clicking the mouse advances to the next effect
def mouse_pressed():
    global effect
    effect += 1
    if effect >= len(effect_names):
        effect = 0

    redraw()


# Moving the mouse triggers a screen redraw
def mouse_moved():
    redraw()


def mouse_dragged():
    redraw()


def draw():
    # We're only going to process a portion of the image
    # so let's set the whole image as the background first
    image(img, 0, 0)

    # Calculate the small rectangle we will process
    xstart = int(constrain(mouse_x - w // 2, 0, img.width))
    ystart = int(constrain(mouse_y - w // 2, 0, img.height))
    xend = int(constrain(mouse_x + w // 2, 0, img.width))
    yend = int(constrain(mouse_y + w // 2, 0, img.height))
    matrixsize = 3
    pixels = load_pixels()
    # Begin our loop for every pixel in the smaller image
    for x in range(xstart, xend):
        for y in range(ystart, yend):
            c = convolution(x, y, kernels[effect], matrixsize, img)
            loc = x + y * img.width
            pixels[loc] = c
    update_pixels(pixels)

    text_size(24)
    text(effect_names[effect], 4, 24)


def convolution(x, y, matrix, matrixsize, img):
    px = img.pixels
    rtotal = 0.0
    gtotal = 0.0
    btotal = 0.0
    offset = matrixsize // 2
    for i in range(matrixsize):
        for j in range(matrixsize):
            # What pixel are we testing
            xloc = x + i - offset
            yloc = y + j - offset
            loc = xloc + img.width * yloc
            # Make sure we haven't walked off our image, we could do better here
            loc = int(constrain(loc, 0, len(px) - 1))
            # Calculate the convolution
            rtotal += px[loc].r * matrix[i][j]
            gtotal += px[loc].g * matrix[i][j]
            btotal += px[loc].b * matrix[i][j]
    # Make sure RGB is within range
    rtotal = constrain(rtotal, 0, 1)
    gtotal = constrain(gtotal, 0, 1)
    btotal = constrain(btotal, 0, 1)
    # Return the resulting color
    return color(rtotal, gtotal, btotal)


run()
