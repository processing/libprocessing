# Load and Display a Shape.
# Illustration by George Brower.
#
# The load_shape() command is used to read simple SVG (Scalable Vector Graphics)
# files into a sketch. This example loads an SVG file of a monster robot face
# and displays it to the screen.
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
    shape(bot, 110, 90, 100, 100)  # GAP: shape(s, x, y, w, h); draw at (110, 90) at size 100 x 100
    shape(bot, 280, 40)  # Draw at coordinate (280, 40) at the default size


run()
