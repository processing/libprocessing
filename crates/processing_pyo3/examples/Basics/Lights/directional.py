# Directional.
#
# Move the mouse the change the direction of the light.
# Directional light comes from one direction and is stronger
# when hitting a surface squarely and weaker if it hits at a
# a gentle angle. After hitting a surface, a directional lights
# scatters in all directions.
from mewnala import *

light = None


def setup():
    global light
    size(640, 360)
    mode_3d()
    no_stroke()
    fill(0.8)
    # Lights are created once; the shapes become lit once a material property is set
    light = directional_light((0.8, 0.8, 0.8), 2500.0)
    roughness(0.6)


def draw():
    no_stroke()
    background(0.0)
    dir_y = (mouse_y / height - 0.5) * 2
    dir_x = (mouse_x / width - 0.5) * 2
    # The world is y-up, so the vertical direction is flipped
    light.look_at(-dir_x, dir_y, -1)
    translate(-100, 0, 0)
    sphere(80)
    translate(200, 0, 0)
    sphere(80)


run()
