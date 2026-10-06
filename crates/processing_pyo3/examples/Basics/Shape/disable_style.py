# Disable Style
# by George Brower.
#
# Shapes are loaded with style information that tells them how
# to draw (e.g. color, stroke weight). The disable_style()
# method of Shape turns off this information so functions like
# stroke() and fill() change the SVGs color. The enable_style()
# method turns the file's original styles back on.
from mewnala import *

bot = None


def setup():
    global bot
    size(640, 360)
    # The file "bot1.svg" must be in the data folder
    # of the current sketch to load successfully
    bot = load_shape("data/bot1.svg")  # GAP: load_shape() (SVG) is missing
    no_loop()


def draw():
    background(0.4)

    # Draw left bot
    bot.disable_style()  # GAP: Shape.disable_style(); ignore the colors in the SVG
    fill(0.0, 0.4, 0.6)  # Set the SVG fill to blue
    stroke(1.0)  # Set the SVG stroke to white
    shape(bot, 20, 25, 300, 300)  # GAP: shape(s, x, y, w, h)

    # Draw right bot
    bot.enable_style()  # GAP: Shape.enable_style()
    shape(bot, 320, 25, 300, 300)


run()
