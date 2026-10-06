# Mouse Functions.
#
# Click on the box and drag it across the screen.
from mewnala import *

bx = 0.0
by = 0.0
box_size = 75
over_box = False
locked = False
x_offset = 0.0
y_offset = 0.0


def setup():
    global bx, by
    size(640, 360)
    bx = width / 2.0
    by = height / 2.0
    rect_mode(RADIUS)


def draw():
    global over_box
    background(0.0)

    # Test if the cursor is over the box
    if (mouse_x > bx - box_size and mouse_x < bx + box_size and
            mouse_y > by - box_size and mouse_y < by + box_size):
        over_box = True
        if not locked:
            stroke(1.0)
            fill(0.6)
    else:
        stroke(0.6)
        fill(0.6)
        over_box = False

    # Draw the box
    rect(bx, by, box_size, box_size)


def mouse_pressed():
    global locked, x_offset, y_offset
    if over_box:
        locked = True
        fill(1.0, 1.0, 1.0)
    else:
        locked = False
    x_offset = mouse_x - bx
    y_offset = mouse_y - by


def mouse_dragged():
    global bx, by
    if locked:
        bx = mouse_x - x_offset
        by = mouse_y - y_offset


def mouse_released():
    global locked
    locked = False


run()
