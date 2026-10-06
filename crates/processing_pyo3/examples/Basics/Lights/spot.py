# Spot.
#
# Move the mouse the change the position and concentation
# of a blue spot light.
from mewnala import *

bottom = None
orange = None
blue = None


def setup():
    global bottom, orange, blue
    size(640, 360)
    mode_3d()
    no_stroke()
    fill(0.8)

    # Lights are created once and positioned relative to the top-left corner
    # as in the original (the world is centered and y-up)
    # Light the bottom of the sphere
    bottom = directional_light((0.2, 0.4, 0.49), 4000.0, look_at=(0, 1, 0))

    # Orange light on the upper-right of the sphere
    orange = spot_light((0.8, 0.6, 0.0), 10000000000.0, 1200.0, 1.0, 0.0, PI / 12,
                        position=(360 - width / 2, height / 2 - 160, 600),
                        look_at=(360 - width / 2, height / 2 - 160, 0))

    # Moving spotlight that follows the mouse
    blue = spot_light((0.4, 0.6, 0.8), 10000000000.0, 1200.0, 1.0, 0.0, PI / 12,
                      position=(360 - width / 2, height / 2, 600),
                      look_at=(360 - width / 2, height / 2, 0))
    roughness(0.6)


def draw():
    background(0.0)

    blue.position(360 - width / 2, height / 2 - mouse_y, 600)
    blue.look_at(360 - width / 2, height / 2 - mouse_y, 0)

    sphere(120, 60, 60)


run()
