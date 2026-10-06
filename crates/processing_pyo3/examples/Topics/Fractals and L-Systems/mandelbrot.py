# The Mandelbrot Set
# by Daniel Shiffman.
# (slight modification by l8l)
#
# Simple rendering of the Mandelbrot set.
from mewnala import *


def setup():
    size(640, 360)
    pixel_density(1)
    no_loop()
    background(1.0)


def draw():
    # Establish a range of values on the complex plane
    # A different range will allow us to "zoom" in or out on the fractal

    # It all starts with the width, try higher or lower values
    w = 4
    h = (w * height) / width

    # Start at negative half the width and height
    xmin = -w / 2
    ymin = -h / 2

    # Make sure we can write to the pixels[] array.
    # Only need to do this once since we don't do any other drawing.
    px = load_pixels()

    # Maximum number of iterations for each point on the complex plane
    maxiterations = 100

    # x goes from xmin to xmax
    xmax = xmin + w
    # y goes from ymin to ymax
    ymax = ymin + h

    # Calculate amount we increment x,y for each pixel
    dx = (xmax - xmin) / width
    dy = (ymax - ymin) / height

    # Start y
    y = ymin
    for j in range(height):
        # Start x
        x = xmin
        for i in range(width):

            # Now we test, as we iterate z = z^2 + c does z tend towards infinity?
            a = x
            b = y
            n = 0
            max_abs = 4.0  # Infinity in our finite world is simple, let's just consider it 4
            abs_old = 0.0
            converge_number = maxiterations  # this will change if the while loop breaks due to non-convergence
            while n < maxiterations:
                # We suppose z = a+ib
                aa = a * a
                bb = b * b
                abs_z = sqrt(aa + bb)
                if abs_z > max_abs:  # |z| = sqrt(a^2+b^2)
                    # Now measure how much we exceeded the maximum:
                    diff_to_last = abs_z - abs_old
                    diff_to_max = max_abs - abs_old
                    converge_number = n + diff_to_max / diff_to_last
                    break  # Bail
                twoab = 2.0 * a * b
                a = aa - bb + x  # this operation corresponds to z -> z^2+c where z=a+ib c=(x,y)
                b = twoab + y
                n += 1
                abs_old = abs_z

            # We color each pixel based on how long it takes to get to infinity
            # If we never got there, let's pick the color black
            if n == maxiterations:
                px[i + j * width] = color(0.0)
            else:
                # Gosh, we could make fancy colors here if we wanted
                norm_n = remap(converge_number, 0, maxiterations, 0, 1)
                px[i + j * width] = color(sqrt(norm_n))
            x += dx
        y += dy
    update_pixels(px)


run()
