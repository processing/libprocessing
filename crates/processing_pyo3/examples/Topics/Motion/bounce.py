# Bounce.
#
# When the shape hits the edge of the window, it reverses its direction.
from mewnala import *

rad = 60  # Width of the shape
xpos = 0.0  # Starting position of shape
ypos = 0.0

xspeed = 2.8  # Speed of the shape
yspeed = 2.2  # Speed of the shape

xdirection = 1  # Left or Right
ydirection = 1  # Top to Bottom


def setup():
    global xpos, ypos
    size(640, 360)
    no_stroke()
    frame_rate(30)
    ellipse_mode(RADIUS)
    # Set the starting position of the shape
    xpos = width / 2
    ypos = height / 2


def draw():
    global xpos, ypos, xdirection, ydirection
    background(0.4)

    # Update the position of the shape
    xpos = xpos + (xspeed * xdirection)
    ypos = ypos + (yspeed * ydirection)

    # Test to see if the shape exceeds the boundaries of the screen
    # If it does, reverse its direction by multiplying by -1
    if xpos > width - rad or xpos < rad:
        xdirection *= -1
    if ypos > height - rad or ypos < rad:
        ydirection *= -1

    # Draw the shape
    ellipse(xpos, ypos, rad, rad)


run()
