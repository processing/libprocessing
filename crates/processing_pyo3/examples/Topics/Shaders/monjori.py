# Monjori.
#
# GLSL version of the 1k intro Monjori from the demoscene
# (http://www.pouet.net/prod.php?which=52761)
# Ported from the webGL version available in ShaderToy:
# http://www.iquilezles.org/apps/shadertoy/
# (Look for Monjori under the Plane Deformations Presets)
from mewnala import *

monjori = None


def setup():
    global monjori
    size(640, 360)
    no_stroke()

    monjori = load_shader("data/monjori.wesl")


def draw():
    # This kind of effects are entirely implemented in the
    # fragment shader; as a filter every pixel of the canvas
    # is pushed through the shader.
    filter(monjori, time=millis() / 1000.0)


run()
