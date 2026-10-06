# Brightness
# by Rusty Robison.
#
# Brightness is the relative lightness or darkness of a color.
# Move the cursor vertically over each bar to alter its brightness.
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
        fill(hsva(bar_x / width * 360, 1.0, mouse_y / height))
        rect(bar_x, 0, bar_width, height)
        last_bar = which_bar


run()
