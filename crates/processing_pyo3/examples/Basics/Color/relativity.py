# Relativity.
#
# Each color is perceived in relation to other colors. The top and bottom
# bars each contain the same component colors, but a different display order
# causes individual colors to appear differently.
from mewnala import *

a = None
b = None
c = None
d = None
e = None


def setup():
    global a, b, c, d, e
    size(640, 360)
    no_stroke()
    a = color(0.65, 0.65, 0.08)
    b = color(0.3, 0.34, 0.23)
    c = color(0.16, 0.42, 0.41)
    d = color(0.65, 0.35, 0.08)
    e = color(0.57, 0.59, 0.5)
    no_loop()  # Draw only one time


def draw():
    draw_band(a, b, c, d, e, 0, width // 128)
    draw_band(c, a, d, b, e, height // 2, width // 128)


def draw_band(v, w, x, y, z, ypos, bar_width):
    num = 5
    color_order = [v, w, x, y, z]
    for i in range(0, width, bar_width * num):
        for j in range(num):
            fill(color_order[j])
            rect(i + j * bar_width, ypos, bar_width, height // 2)


run()
