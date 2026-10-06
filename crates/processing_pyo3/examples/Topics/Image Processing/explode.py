# Explode
# by Daniel Shiffman.
#
# Mouse horizontal location controls breaking apart of image and
# Maps pixels from a 2D image into 3D space. Pixel brightness controls
# translation along z axis.
from mewnala import *

img = None  # The source image
cellsize = 2  # Dimensions of each cell in the grid
columns = 0  # Number of columns and rows in our system
rows = 0


def setup():
    global img, columns, rows
    size(640, 360)
    mode_3d()
    img = load_image("data/eames.jpg")  # Load the image
    flush()  # make the image's pixels readable right away
    columns = img.width // cellsize  # Calculate # of columns
    rows = img.height // cellsize  # Calculate # of rows


def draw():
    background(0.0)
    px = img.pixels
    # Begin loop for columns
    for i in range(columns):
        # Begin loop for rows
        for j in range(rows):
            x = i * cellsize + cellsize // 2  # x position
            y = j * cellsize + cellsize // 2  # y position
            loc = x + y * img.width  # Pixel array location
            c = px[loc]  # Grab the color
            # Calculate a z position as a function of mouse_x and pixel brightness (0..255 pixels)
            z = (mouse_x / width) * brightness(px[loc]) * 255 - 20.0
            # Translate to the location, set fill and stroke, and draw the rect
            push_matrix()
            translate(x + 200 - width / 2, height / 2 - (y + 100), z)
            fill(c.with_alpha(0.8))
            no_stroke()
            rect_mode(CENTER)
            rect(0, 0, cellsize, cellsize)
            pop_matrix()


run()
