# Redraw Test
#
# no_loop() stops the draw loop; every key press calls redraw()
# to draw one more frame.
from mewnala import *


def setup():
    size(400, 400)
    mode_3d()
    no_loop()


def draw():
    background(1.0, 0.0, 0.0)
    ellipse(mouse_x - width / 2, height / 2 - mouse_y, 100, 50)
    print("draw")


def key_pressed():
    redraw()


run()
