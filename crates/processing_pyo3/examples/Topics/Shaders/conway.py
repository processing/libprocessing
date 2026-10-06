# GLSL version of Conway's game of life, ported from GLSL sandbox:
# http://glsl.heroku.com/e#207.3
# Exemplifies the use of the previous frame's pixels in the shader: a
# filter samples the canvas as it was before the filter ran.
from mewnala import *

conway = None
pg = None


def setup():
    global conway, pg
    size(400, 400)
    pg = create_graphics(400, 400)
    # the world starts dead; each frame the filter reads this previous state
    pg.begin_draw()
    pg.background(0.0)
    pg.end_draw()
    conway = load_shader("data/conway.wesl")


def draw():
    # the filter's uv has y pointing down, like mouse_y
    x = remap(mouse_x, 0, width, 0, 1)
    y = remap(mouse_y, 0, height, 0, 1)
    pg.begin_draw()
    pg.filter(conway, time=millis() / 1000.0, mouse=(x, y))
    pg.end_draw()
    background(0.0)
    image(pg, 0, 0, width, height)


run()
