# Texture Triangle.
#
# Using a rectangular image to map a texture onto a triangle.
from mewnala import *

img = None
tri = None


def setup():
    global img, tri
    size(640, 360)
    mode_3d()
    img = load_image("data/berlin-1.jpg")
    no_stroke()

    # uv coordinates are the image coordinates divided by the image size,
    # and the world is y-up (the top edge is +100)
    tri = create_geometry(topology=TRIANGLES)
    tri.uv(0, 0)
    tri.vertex(-100, 100, 0)
    tri.uv(300 / img.width, 120 / img.height)
    tri.vertex(100, 40, 0)
    tri.uv(200 / img.width, 400 / img.height)
    tri.vertex(0, -100, 0)


def draw():
    background(0.0)
    rotate_y(remap(mouse_x, 0, width, -PI, PI))
    unlit()
    texture(img)
    draw_geometry(tri)
    no_texture()


run()
