# Constrain.
#
# Move the mouse across the screen to move the circle.
# The program constrains the circle to its box.
from mewnala import *

mx = 0.0
my = 0.0
easing = 0.05
radius = 24
edge = 100
inner = edge + radius


def setup():
    size(640, 360)
    no_stroke()
    ellipse_mode(RADIUS)
    rect_mode(CORNERS)


def draw():
    global mx, my
    background(0.2)

    if abs(mouse_x - mx) > 0.1:
        mx = mx + (mouse_x - mx) * easing
    if abs(mouse_y - my) > 0.1:
        my = my + (mouse_y - my) * easing

    mx = constrain(mx, inner, width - inner)
    my = constrain(my, inner, height - inner)
    fill(0.3)
    rect(edge, edge, width - edge, height - edge)
    fill(1.0)
    ellipse(mx, my, radius, radius)


run()
