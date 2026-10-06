# Words.
#
# The text() function is used for writing words to the screen.
# The letters can be aligned left, center, or right with the
# text_align() function.
from mewnala import *

f = None


def setup():
    global f
    size(640, 360)

    # Create the font
    print(list_fonts())
    f = load_font("data/SpaceMono-Regular.ttf")
    text_font(f)
    text_size(18)


def draw():
    background(0.4)
    text_align(RIGHT)
    draw_type(width * 0.25)
    text_align(CENTER)
    draw_type(width * 0.5)
    text_align(LEFT)
    draw_type(width * 0.75)


def draw_type(x):
    line(x, 0, x, 65)
    line(x, 220, x, height)
    fill(0.0)
    text("ichi", x, 95)
    fill(0.2)
    text("ni", x, 130)
    fill(0.8)
    text("san", x, 165)
    fill(1.0)
    text("shi", x, 210)


run()
