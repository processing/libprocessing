# Scale Shape.
# Illustration by George Brower.
#
# Move the mouse left and right to zoom the SVG file.
# This shows how, unlike an imported image, the lines
# remain smooth at any size.
from mewnala import *

bot = None


def setup():
    global bot
    size(640, 360)
    # The file "bot1.svg" must be in the data folder
    # of the current sketch to load successfully
    bot = load_shape("data/bot1.svg")  # GAP: load_shape() (SVG) is missing


def draw():
    background(0.4)
    translate(width / 2, height / 2)
    zoom = remap(mouse_x, 0, width, 0.1, 4.5)
    scale(zoom)
    shape(bot, -140, -140)  # GAP: shape(s, x, y)


run()
