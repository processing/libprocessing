# Nebula.
#
# From CoffeeBreakStudios.com (CBS)
# Ported from the webGL version in GLSL Sandbox:
# http://glsl.heroku.com/e#3265.2
from mewnala import *

nebula = None


def setup():
    global nebula
    size(640, 360)
    no_stroke()

    nebula = load_shader("data/nebula.wesl")


def draw():
    # This kind of raymarching effects are entirely implemented in the
    # fragment shader; as a filter every pixel of the canvas is pushed
    # through the shader.
    filter(nebula, time=millis() / 500.0)


run()
