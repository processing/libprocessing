# Vertices
# by Simon Greenwold.
#
# Draw a cylinder centered on the y-axis, going down
# from y=0 to y=height. The radius at the top can be
# different from the radius at the bottom, and the
# number of sides drawn is variable.
from mewnala import *

light = None
cylinder = None


def setup():
    global light, cylinder
    size(640, 360)
    mode_3d()
    light = directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)
    cylinder = build_cylinder(10, 180, 200, 16)  # Build a mix between a cylinder and a cone
    # cylinder = build_cylinder(70, 70, 120, 64)  # Build a cylinder
    # cylinder = build_cylinder(0, 180, 200, 4)  # Build a pyramid


def draw():
    background(0.0)
    rotate_y(remap(mouse_x, 0, width, 0, PI))
    rotate_z(remap(mouse_y, 0, height, 0, -PI))
    no_stroke()
    fill(1.0, 1.0, 1.0)
    # The world is y-up: the cylinder hangs down from y=40
    translate(0, 40, 0)
    draw_geometry(cylinder)


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


def build_cylinder(top_radius, bottom_radius, tall, sides):
    g = create_geometry(topology=TRIANGLES)
    angle = 0
    angle_increment = TWO_PI / sides
    # The sides: one quad per step around the axis (going down to y=-tall)
    for i in range(sides):
        a0 = angle
        a1 = angle + angle_increment
        add_quad(g,
                 Vec3(top_radius * cos(a0), 0, top_radius * sin(a0)),
                 Vec3(top_radius * cos(a1), 0, top_radius * sin(a1)),
                 Vec3(bottom_radius * cos(a1), -tall, bottom_radius * sin(a1)),
                 Vec3(bottom_radius * cos(a0), -tall, bottom_radius * sin(a0)))
        angle += angle_increment

    # If it is not a cone, draw the circular top cap as a fan around the center
    if top_radius != 0:
        angle = 0
        center = Vec3(0, 0, 0)
        for i in range(sides):
            add_triangle(g, center,
                         Vec3(top_radius * cos(angle + angle_increment), 0, top_radius * sin(angle + angle_increment)),
                         Vec3(top_radius * cos(angle), 0, top_radius * sin(angle)))
            angle += angle_increment

    # If it is not a cone, draw the circular bottom cap
    if bottom_radius != 0:
        angle = 0
        center = Vec3(0, -tall, 0)
        for i in range(sides):
            add_triangle(g, center,
                         Vec3(bottom_radius * cos(angle), -tall, bottom_radius * sin(angle)),
                         Vec3(bottom_radius * cos(angle + angle_increment), -tall, bottom_radius * sin(angle + angle_increment)))
            angle += angle_increment
    return g


run()
