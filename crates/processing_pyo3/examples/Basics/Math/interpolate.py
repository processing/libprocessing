# Linear Interpolation.
#
# Move the mouse across the screen and the symbol will follow.
# Between drawing each frame of the animation, the ellipse moves
# part of the distance (0.05) from its current position toward
# the cursor using the lerp() function.
from mewnala import *

x = 0.0
y = 0.0


def setup():
    size(640, 360)
    no_stroke()


def draw():
    global x, y
    background(0.2)

    # lerp() calculates a number between two numbers at a specific increment.
    # The amt parameter is the amount to interpolate between the two values
    # where 0.0 equal to the first point, 0.1 is very near the first point, 0.5
    # is half-way in between, etc.

    # Here we are moving 5% of the way to the mouse location each frame
    x = lerp(x, mouse_x, 0.05)
    y = lerp(y, mouse_y, 0.05)

    fill(1.0)
    stroke(1.0)
    ellipse(x, y, 66, 66)


run()
