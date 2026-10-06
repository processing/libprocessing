# Random Gaussian.
#
# This sketch draws ellipses with x and y locations tied to a gaussian distribution of random numbers.
from mewnala import *


def setup():
    size(640, 360)
    background(0.0)


def draw():
    # Get a gaussian random number w/ mean of 0 and standard deviation of 1.0
    val = random_gaussian()

    sd = 60  # Define a standard deviation
    mean = width / 2  # Define a mean value (middle of the screen along the x-axis)
    x = (val * sd) + mean  # Scale the gaussian random number by standard deviation and mean

    no_stroke()
    fill(1.0, 0.04)
    ellipse(x, height / 2, 32, 32)  # Draw an ellipse at our "normal" random location


run()
