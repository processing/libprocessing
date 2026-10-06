# Moving On Curves.
#
# In this example, the circles moves along the curve y = x^4.
# Click the mouse to have it move to a new position.
from mewnala import *

begin_x = 20.0  # Initial x-coordinate
begin_y = 10.0  # Initial y-coordinate
end_x = 570.0  # Final x-coordinate
end_y = 320.0  # Final y-coordinate
dist_x = 0.0  # X-axis distance to move
dist_y = 0.0  # Y-axis distance to move
exponent = 4  # Determines the curve
x = 0.0  # Current x-coordinate
y = 0.0  # Current y-coordinate
step = 0.01  # Size of each step along the path
pct = 0.0  # Percentage traveled (0.0 to 1.0)


def setup():
    global dist_x, dist_y
    size(640, 360)
    no_stroke()
    dist_x = end_x - begin_x
    dist_y = end_y - begin_y


def draw():
    global pct, x, y
    fill(0.0, 0.01)
    rect(0, 0, width, height)
    pct += step
    if pct < 1.0:
        x = begin_x + (pct * dist_x)
        y = begin_y + (pow(pct, exponent) * dist_y)
    fill(1.0)
    ellipse(x, y, 20, 20)


def mouse_pressed():
    global pct, begin_x, begin_y, end_x, end_y, dist_x, dist_y
    pct = 0.0
    begin_x = x
    begin_y = y
    end_x = mouse_x
    end_y = mouse_y
    dist_x = end_x - begin_x
    dist_y = end_y - begin_y


run()
