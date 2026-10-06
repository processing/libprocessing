# Pulses.
#
# Software drawing instruments can follow a rhythm or abide by rules independent
# of drawn gestures. This is a form of collaborative drawing in which the draftsperson
# controls some aspects of the image and the software controls others.
from mewnala import *

angle = 0


def setup():
    size(640, 360)
    background(0.4)
    no_stroke()
    fill(0.0, 0.4)


def draw():
    global angle
    # Draw only when mouse is pressed
    if mouse_is_pressed:
        angle += 5
        val = cos(radians(angle)) * 12.0
        for a in range(0, 360, 75):
            xoff = cos(radians(a)) * val
            yoff = sin(radians(a)) * val
            fill(0.0)
            ellipse(mouse_x + xoff, mouse_y + yoff, val, val)
        fill(1.0)
        ellipse(mouse_x, mouse_y, 2, 2)


run()
