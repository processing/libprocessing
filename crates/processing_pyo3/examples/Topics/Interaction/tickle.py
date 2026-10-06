# Tickle.
#
# The word "tickle" jitters when the cursor hovers over.
# Sometimes, it can be tickled off the screen.
from mewnala import *

message = "tickle"
x = 0.0  # X and Y coordinates of text
y = 0.0
hr = 0.0  # horizontal and vertical radius of the text
vr = 0.0


def setup():
    global x, y, hr, vr
    size(640, 360)

    # Create the font
    text_font(load_font("data/SourceCodePro-Regular.ttf"))
    text_size(36)
    text_align(CENTER, CENTER)

    hr = text_width(message) / 2
    vr = (text_ascent() + text_descent()) / 2
    no_stroke()
    x = width / 2
    y = height / 2


def draw():
    global x, y
    # Instead of clearing the background, fade it by drawing
    # a semi-transparent rectangle on top
    fill(0.8, 0.47)
    rect(0, 0, width, height)

    # If the cursor is over the text, change the position
    if abs(mouse_x - x) < hr and abs(mouse_y - y) < vr:
        x += random(-5, 5)
        y += random(-5, 5)
    fill(0.0)
    text("tickle", x, y)


run()
