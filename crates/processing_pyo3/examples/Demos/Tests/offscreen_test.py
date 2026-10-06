# Offscreen Test
#
# Draws into an offscreen buffer and displays it on the main canvas.
from mewnala import *

pg = None


def setup():
    global pg
    size(400, 400)
    mode_3d()

    pg = create_graphics(400, 400)


def draw():
    background(0.0)

    pg.begin_draw()
    pg.background(1.0, 0.0, 0.0)
    pg.ellipse(mouse_x, mouse_y, 100, 100)
    pg.end_draw()

    image(pg, -width / 2, -height / 2, 400, 400)


run()
