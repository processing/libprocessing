# Texture Quad.
#
# Load an image and draw it onto a quad. The texture() function sets
# the texture image. The uv() function maps the image to the geometry.
from mewnala import *

img = None
quad = None


def setup():
    global img, quad
    size(640, 360)
    mode_3d()
    img = load_image("data/berlin-1.jpg")
    no_stroke()

    # uv coordinates are the image coordinates divided by the image size,
    # and the world is y-up (the top edge is +100)
    quad = create_geometry(topology=TRIANGLES)
    quad.uv(0, 0)
    quad.vertex(-100, 100, 0)
    quad.uv(1, 0)
    quad.vertex(100, 100, 0)
    quad.uv(1, 1)
    quad.vertex(100, -100, 0)
    quad.uv(0, 1)
    quad.vertex(-100, -100, 0)
    for i in (0, 1, 2, 0, 2, 3):
        quad.index(i)


def draw():
    background(0.0)
    rotate_y(remap(mouse_x, 0, width, -PI, PI))
    # Rotations about z run the other way in the y-up world
    rotate_z(-PI / 6)
    unlit()
    texture(img)
    draw_geometry(quad)
    no_texture()


run()
