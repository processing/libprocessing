# Loading URLs.
#
# Click on the button to open a URL in a browser.
from mewnala import *

over_button = False


def setup():
    size(640, 360)


def draw():
    background(0.8)

    if over_button:
        fill(1.0)
    else:
        no_fill()
    rect(105, 60, 75, 75)
    line(135, 105, 155, 85)
    line(140, 85, 155, 85)
    line(155, 85, 155, 100)


def mouse_pressed():
    if over_button:
        link("http://www.processing.org")  # GAP: link(url) (open a URL in the browser) is missing


def mouse_moved():
    check_buttons()


def mouse_dragged():
    check_buttons()


def check_buttons():
    global over_button
    if mouse_x > 105 and mouse_x < 180 and mouse_y > 60 and mouse_y < 135:
        over_button = True
    else:
        over_button = False


run()
