# Brownian motion.
#
# Recording random movement as a continuous line.
from mewnala import *

num = 2000
step_range = 6

ax = []
ay = []


def setup():
    size(640, 360)
    for i in range(num):
        ax.append(width / 2)
        ay.append(height / 2)
    frame_rate(30)


def draw():
    background(0.2)

    # Shift all elements 1 place to the left
    for i in range(1, num):
        ax[i - 1] = ax[i]
        ay[i - 1] = ay[i]

    # Put a new value at the end of the array
    ax[num - 1] += random(-step_range, step_range)
    ay[num - 1] += random(-step_range, step_range)

    # Constrain all points to the screen
    ax[num - 1] = constrain(ax[num - 1], 0, width)
    ay[num - 1] = constrain(ay[num - 1], 0, height)

    # Draw a line connecting the points
    for i in range(1, num):
        val = i / num * 0.8 + 0.2
        stroke(val)
        line(ax[i - 1], ay[i - 1], ax[i], ay[i])


run()
