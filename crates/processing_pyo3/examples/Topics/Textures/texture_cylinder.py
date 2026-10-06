# Texture Cylinder.
#
# Load an image and draw it onto a cylinder and a quad.
from mewnala import *

tube_res = 32
tube_x = []
tube_y = []
img = None
tube = None
quad = None


def setup():
    global img, tube, quad
    size(640, 360)
    mode_3d()
    img = load_image("data/berlin-1.jpg")
    angle = 270.0 / tube_res
    for i in range(tube_res):
        tube_x.append(cos(radians(i * angle)))
        tube_y.append(sin(radians(i * angle)))
    no_stroke()

    # The meshes are built once; uv coordinates are the image coordinates
    # divided by the image size, and the world is y-up (the top edge is +100)
    tube = create_geometry(topology=TRIANGLE_STRIP)
    for i in range(tube_res):
        x = tube_x[i] * 100
        z = tube_y[i] * 100
        u = (img.width // tube_res * i) / img.width
        tube.uv(u, 0)
        tube.vertex(x, 100, z)
        tube.uv(u, 1)
        tube.vertex(x, -100, z)

    quad = create_geometry(topology=TRIANGLES)
    quad.uv(0, 0)
    quad.vertex(0, 100, 0)
    quad.uv(100 / img.width, 0)
    quad.vertex(100, 100, 0)
    quad.uv(100 / img.width, 100 / img.height)
    quad.vertex(100, -100, 0)
    quad.uv(0, 100 / img.height)
    quad.vertex(0, -100, 0)
    for i in (0, 1, 2, 0, 2, 3):
        quad.index(i)


def draw():
    background(0.0)
    rotate_x(remap(mouse_y, 0, height, -PI, PI))
    rotate_y(remap(mouse_x, 0, width, -PI, PI))
    unlit()
    texture(img)
    draw_geometry(tube)
    draw_geometry(quad)
    no_texture()


run()
