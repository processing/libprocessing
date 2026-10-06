# Shape Transform
# by Ira Greenberg.
#
# Illustrates the geometric relationship
# between Cube, Pyramid, Cone and
# Cylinder 3D primitives.
#
# Instructions:
# Up Arrow - increases points
# Down Arrow - decreases points
# 'p' key toggles between cube/pyramid
from mewnala import *

pts = 4
angle = 0
radius = 99
cylinder_length = 95

# vertices
vertices = []
is_pyramid = False

angle_inc = 0
light = None
tube = None
ends = None


def setup():
    global angle_inc, light
    size(640, 360)
    mode_3d()
    no_stroke()
    angle_inc = PI / 300.0
    light = directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)
    build_shape()


def draw():
    background(0.67, 0.37, 0.37)
    fill(1.0, 0.78, 0.78)
    rotate_x(frame_count * angle_inc)
    rotate_y(frame_count * angle_inc)
    rotate_z(frame_count * angle_inc)
    draw_geometry(tube)
    draw_geometry(ends)


# The mesh only changes with the keys, so it is rebuilt on key presses
def build_shape():
    global vertices, angle, cylinder_length, tube, ends
    # initialize vertex arrays
    vertices = [[], []]

    # fill arrays
    for i in range(2):
        angle = 0
        for j in range(pts + 1):
            v = Vec3(0, 0, 0)
            if is_pyramid:
                if i == 1:
                    v.x = 0
                    v.y = 0
                else:
                    v.x = cos(radians(angle)) * radius
                    v.y = sin(radians(angle)) * radius
            else:
                v.x = cos(radians(angle)) * radius
                v.y = sin(radians(angle)) * radius
            v.z = cylinder_length
            vertices[i].append(v)
            angle += 360.0 / pts
        cylinder_length *= -1

    # build cylinder tube
    tube = create_geometry(topology=TRIANGLES)
    for j in range(pts):
        add_quad(tube, vertices[0][j], vertices[1][j], vertices[1][j + 1], vertices[0][j + 1])

    # build cylinder ends as fans; the two ends face opposite ways
    ends = create_geometry(topology=TRIANGLES)
    for j in range(1, pts - 1):
        add_triangle(ends, vertices[0][0], vertices[0][j], vertices[0][j + 1])
        add_triangle(ends, vertices[1][0], vertices[1][j + 1], vertices[1][j])


# Adds a flat-shaded triangle: lit geometry needs a normal per vertex
def add_triangle(g, a, b, c):
    n = (b - a).cross(c - a)
    n.normalize()
    g.normal(n.x, n.y, n.z)
    g.vertex(a.x, a.y, a.z)
    g.vertex(b.x, b.y, b.z)
    g.vertex(c.x, c.y, c.z)


def add_quad(g, a, b, c, d):
    add_triangle(g, a, b, c)
    add_triangle(g, a, c, d)


# up/down arrow keys control
# polygon detail.
def key_pressed():
    global pts, is_pyramid
    if key is None:
        # pts
        if key_code == UP:
            if pts < 90:
                pts += 1
        elif key_code == DOWN:
            if pts > 4:
                pts -= 1
    if key == "p":
        if is_pyramid:
            is_pyramid = False
        else:
            is_pyramid = True
    build_shape()


run()
