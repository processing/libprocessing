from mewnala import *

geometry = None


def setup():
    global geometry
    size(640, 480)
    mode_3d()

    geometry = Geometry()

    geometry.normal(0.0, 0.0, 1.0)

    geometry.uv(0.0, 0.0)
    geometry.vertex(-80.0, -80.0, 0.0)

    geometry.uv(1.0, 0.0)
    geometry.vertex(80.0, -80.0, 0.0)

    geometry.uv(0.5, 1.0)
    geometry.vertex(0.0, 80.0, 0.0)

    geometry.index(0)
    geometry.index(1)
    geometry.index(2)

    print("vertex_count =", geometry.vertex_count())


def draw():
    background(220)
    camera_position(0.0, 0.0, 200.0)
    camera_look_at(0.0, 0.0, 0.0)
    draw_geometry(geometry)


run()
