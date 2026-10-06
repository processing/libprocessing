# Sine Wave
# by Daniel Shiffman.
#
# Render a simple sine wave.
from mewnala import *

xspacing = 16  # How far apart should each horizontal location be spaced
w = 0  # Width of entire wave

theta = 0.0  # Start angle at 0
amplitude = 75.0  # Height of wave
period = 500.0  # How many pixels before the wave repeats
dx = 0.0  # Value for incrementing X, a function of period and xspacing
yvalues = []  # Using an array to store height values for the wave


def setup():
    global w, dx, yvalues
    size(640, 360)
    w = width + 16
    dx = (TWO_PI / period) * xspacing
    yvalues = [0.0] * (w // xspacing)


def draw():
    background(0.0)
    calc_wave()
    render_wave()


def calc_wave():
    global theta
    # Increment theta (try different values for 'angular velocity' here
    theta += 0.02

    # For every x value, calculate a y value with sine function
    x = theta
    for i in range(len(yvalues)):
        yvalues[i] = sin(x) * amplitude
        x += dx


def render_wave():
    no_stroke()
    fill(1.0)
    # A simple way to draw the wave with an ellipse at each location
    for x in range(len(yvalues)):
        ellipse(x * xspacing, height / 2 + yvalues[x], 16, 16)


run()
