# Reflection
# by Simon Greenwold.
#
# Vary the specular reflection component of a material
# with the horizontal position of the mouse.
from processing_py import *


def setup():
    size(640, 360)
    mode_3d()
    no_stroke()
    fill(0.4)


def draw():
    directional_light((0.8, 0.8, 0.8), 4000.0, look_at=(0, 0, -1))
    background(0.0)
    # A smoother (less rough) material gives a stronger specular highlight
    s = mouse_x / width
    roughness(1.0 - s * 0.6)
    sphere(120)


run()
