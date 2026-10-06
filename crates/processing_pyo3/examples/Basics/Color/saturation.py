# Saturation.
#
# Saturation is the strength or purity of the color and represents the
# amount of gray in proportion to the hue. A "saturated" color is pure
# and an "unsaturated" color has a large percentage of gray.
# Move the cursor vertically over each bar to alter its saturation.
from mewnala import *

bar_width = 20
last_bar = -1


def setup():
    size(640, 360)
    no_stroke()


def draw():
    global last_bar
    which_bar = int(mouse_x) // bar_width
    if which_bar != last_bar:
        bar_x = which_bar * bar_width
        fill(hsva(bar_x / width * 360, mouse_y / height, 0.66))
        rect(bar_x, 0, bar_width, height)
        last_bar = which_bar


run()
