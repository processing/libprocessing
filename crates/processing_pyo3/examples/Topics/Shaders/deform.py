# Deform.
#
# A GLSL version of the oldschool 2D deformation effect, by Inigo Quilez.
# Ported from the webGL version available in ShaderToy:
# http://www.iquilezles.org/apps/shadertoy/
# (Look for Deform under the Plane Deformations Presets)
from mewnala import *

tex = None
deform = None


def setup():
    global tex, deform
    size(640, 360)

    tex = load_image("data/tex1.jpg")

    deform = load_shader("data/deform.wesl")


def draw():
    # The filter samples the texture itself (as `tex`), so nothing
    # has to be drawn before it.
    filter(deform, tex=tex, time=millis() / 1000.0, mouse=(float(mouse_x), float(mouse_y)))


run()
