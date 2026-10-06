# Interactive Toroid
# by Ira Greenberg.
#
# Illustrates the geometric relationship between Toroid, Sphere, and Helix
# 3D primitives, as well as lathing principal.
#
# Instructions:
# UP arrow key pts++
# DOWN arrow key pts--
# LEFT arrow key segments--
# RIGHT arrow key segments++
# 'a' key toroid radius--
# 's' key toroid radius++
# 'z' key initial polygon radius--
# 'x' key initial polygon radius++
# 'w' key toggle wireframe/solid shading
# 'h' key toggle sphere/helix
from mewnala import *

pts = 40
angle = 0
radius = 60.0

# lathe segments
segments = 60
lathe_angle = 0
lathe_radius = 100.0

# vertices
vertices = []
vertices2 = []

# for shaded or wireframe rendering
is_wire_frame = False

# for optional helix
is_helix = False
helix_offset = 5.0

light = None
toroid = None


def setup():
    global light
    size(640, 360)
    mode_3d()
    # basic lighting setup
    light = directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)
    build_toroid()


def draw():
    background(0.2, 0.25, 0.16)
    # 2 rendering styles
    # wireframe or solid
    if is_wire_frame:
        stroke(1.0, 1.0, 0.59)
        no_fill()  # GAP: stroke()/no_fill() wireframe outline on draw_geometry
    else:
        no_stroke()
        fill(0.59, 0.76, 0.49)
    # center and spin toroid
    translate(0, 0, -100)

    rotate_x(frame_count * PI / 150)
    rotate_y(frame_count * PI / 170)
    rotate_z(frame_count * PI / 90)

    draw_geometry(toroid)


# The mesh only changes with the keys, so it is rebuilt on key presses
def build_toroid():
    global vertices, vertices2, angle, lathe_angle, toroid
    # initialize point arrays
    vertices = []
    vertices2 = []

    # fill arrays
    for i in range(pts + 1):
        v = Vec3(0, 0, 0)
        vertices.append(v)
        vertices2.append(Vec3(0, 0, 0))
        v.x = lathe_radius + sin(radians(angle)) * radius
        if is_helix:
            v.z = cos(radians(angle)) * radius - (helix_offset * segments) / 2
        else:
            v.z = cos(radians(angle)) * radius
        angle += 360.0 / pts

    # build toroid: each lathe segment is a strip of quads between the
    # previous ring (vertices2) and the new one
    toroid = create_geometry(topology=TRIANGLES)
    lathe_angle = 0
    for i in range(segments + 1):
        previous = []
        current = []
        for j in range(pts + 1):
            if i > 0:
                previous.append(Vec3(vertices2[j].x, vertices2[j].y, vertices2[j].z))
            vertices2[j].x = cos(radians(lathe_angle)) * vertices[j].x
            vertices2[j].y = sin(radians(lathe_angle)) * vertices[j].x
            vertices2[j].z = vertices[j].z
            # optional helix offset
            if is_helix:
                vertices[j].z += helix_offset
            current.append(Vec3(vertices2[j].x, vertices2[j].y, vertices2[j].z))
        if i > 0:
            for j in range(pts):
                add_quad(toroid, previous[j], previous[j + 1], current[j + 1], current[j])
        # create extra rotation for helix
        if is_helix:
            lathe_angle += 720.0 / segments
        else:
            lathe_angle += 360.0 / segments


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


# left/right arrow keys control ellipse detail
# up/down arrow keys control segment detail.
# 'a','s' keys control lathe radius
# 'z','x' keys control ellipse radius
# 'w' key toggles between wireframe and solid
# 'h' key toggles between toroid and helix
def key_pressed():
    global pts, segments, lathe_radius, radius, is_wire_frame, is_helix
    if key is None:
        # pts
        if key_code == UP:
            if pts < 40:
                pts += 1
        elif key_code == DOWN:
            if pts > 3:
                pts -= 1
        # extrusion length
        if key_code == LEFT_ARROW:
            if segments > 3:
                segments -= 1
        elif key_code == RIGHT_ARROW:
            if segments < 80:
                segments += 1
    # lathe radius
    if key == "a":
        if lathe_radius > 0:
            lathe_radius -= 1
    elif key == "s":
        lathe_radius += 1
    # ellipse radius
    if key == "z":
        if radius > 10:
            radius -= 1
    elif key == "x":
        radius += 1
    # wireframe
    if key == "w":
        if is_wire_frame:
            is_wire_frame = False
        else:
            is_wire_frame = True
    # helix
    if key == "h":
        if is_helix:
            is_helix = False
        else:
            is_helix = True
    build_toroid()


run()
