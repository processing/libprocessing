# Create Graphics.
#
# The create_graphics() function creates an offscreen graphics
# buffer with the same drawing API as the main canvas. The
# begin_draw() method is necessary to prepare for drawing and
# end_draw() is necessary to finish. Use this if you need to draw
# into an off-screen graphics buffer or to maintain two
# drawing surfaces with different properties.
from mewnala import *

pg = None


def setup():
    global pg
    size(640, 360)
    pg = create_graphics(400, 200)


def draw():
    fill(0.0, 0.05)
    rect(0, 0, width, height)
    fill(1.0)
    no_stroke()
    ellipse(mouse_x, mouse_y, 60, 60)

    pg.begin_draw()
    pg.background(0.2)
    pg.no_fill()
    pg.stroke(1.0)
    pg.ellipse(mouse_x - 120, mouse_y - 60, 60, 60)
    pg.end_draw()

    # Draw the offscreen buffer to the screen with image()
    image(pg, 120, 60)


run()
