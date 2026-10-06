# Hue.
#
# Hue is the color reflected from or transmitted through an object
# and is typically referred to as the name of the color such as
# red, blue, or yellow. In this example, move the cursor vertically
# over each bar to alter its hue.
from mewnala import *

bar_width = 20
last_bar = -1


def setup():
    size(640, 360)
    no_stroke()
    background(0.0)


def draw():
    global last_bar
    which_bar = int(mouse_x) // bar_width
    if which_bar != last_bar:
        bar_x = which_bar * bar_width
        fill(hsva(mouse_y / height * 360, 1.0, 1.0))
        rect(bar_x, 0, bar_width, height)
        last_bar = which_bar


run()
