# Extrusion.
#
# Converts a flat image into spatial data points and rotates the points
# around the center.
from mewnala import *

a = None
onetime = True
a_pixels = []
values = []
angle = 0.0


def setup():
    global a
    size(640, 360)
    mode_3d()

    for j in range(width):
        a_pixels.append([0] * height)
        values.append([0] * height)
    no_fill()

    # Load the image into a new array
    # Extract the values and store in an array
    a = load_image("data/ystone08.jpg")
    flush()  # make the image's pixels readable right away
    px = a.pixels
    for i in range(a.height):
        for j in range(a.width):
            a_pixels[j][i] = px[i * a.width + j]
            values[j][i] = int(a_pixels[j][i].b * 255)


def draw():
    global angle
    background(0.0)
    translate(0, 0, -height / 2)
    scale(2.0)

    # Update and constrain the angle
    angle += 0.005
    rotate_y(angle)

    # Display the image mass
    for i in range(0, a.height, 4):
        for j in range(0, a.width, 4):
            stroke(values[j][i] / 255, 1.0)
            line(j - a.width / 2, a.height / 2 - i, -values[j][i], j - a.width / 2, a.height / 2 - i, -values[j][i] - 10)  # GAP: 3D line(x1, y1, z1, x2, y2, z2)


run()
