# Display endless moving background using a tile texture.
# Contributed by martiSteiger
from mewnala import *

tile_texture = None
tile_shader = None


def setup():
    global tile_texture
    size(640, 480)
    tile_texture = load_image("data/penrose.jpg")
    load_tile_shader()


def load_tile_shader():
    global tile_shader
    tile_shader = load_shader("data/scroller.wesl")


def draw():
    filter(tile_shader, tile_image=tile_texture, time=millis() / 1000.0)


run()
