# Space Junk
# by Ira Greenberg (zoom suggestion by Danny Greenberg).
#
# Rotating cubes in space using a custom Cube class.
# Color controlled by light sources. Move the mouse left
# and right to zoom.
from mewnala import *

# Used for oveall rotation
angle = 0

# Cube count-lower/raise to test performance
limit = 500

# Array for all cubes
cubes = []

blue_light = None
red_light = None


def setup():
    global blue_light, red_light
    size(640, 360)
    mode_3d()
    background(0.0)
    no_stroke()

    # Instantiate cubes, passing in random vals for size and postion
    for i in range(limit):
        cubes.append(Cube(int(random(-10, 10)), int(random(-10, 10)),
                          int(random(-10, 10)), int(random(-140, 140)),
                          int(random(-140, 140)), int(random(-140, 140))))

    # Set up some different colored lights, placed relative to the top-left
    # corner as in the original (the world is centered and y-up)
    blue_light = point_light((0.2, 0.4, 1.0), 3000000000.0, 600.0, 1.0,
                             position=(65 - width / 2, height / 2 - 60, 100))
    red_light = point_light((0.78, 0.16, 0.24), 3000000000.0, 600.0, 1.0,
                            position=(-65 - width / 2, height / 2 + 60, -150))

    # Raise overall light in scene
    ambient_light(0.27, 0.27, 0.04)  # GAP: ambient_light()
    roughness(0.6)


def draw():
    global angle
    background(0.0)
    fill(0.78)

    # Center geometry in display windwow.
    # you can changlee 3rd argument ('0')
    # to move block group closer(+) / further(-)
    translate(0, 0, -200 + mouse_x * 0.65)

    # Rotate around y and x axes
    rotate_y(radians(angle))
    rotate_x(radians(angle))

    # Draw cubes
    for i in range(len(cubes)):
        cubes[i].draw_cube()

    # Used in rotate function calls above
    angle += 0.2


class Cube:
    # Constructor
    def __init__(self, w, h, d, shift_x, shift_y, shift_z):
        self.w = w
        self.h = h
        self.d = d
        self.shift_x = shift_x
        self.shift_y = shift_y
        self.shift_z = shift_z
        self.geometry = self.build()

    # The cube mesh is built once. It looks more confusing than it
    # really is: it's just a bunch of rectangles, one for each cube face
    def build(self):
        w = self.w
        h = self.h
        d = self.d
        sx = self.shift_x
        sy = self.shift_y
        sz = self.shift_z
        g = create_geometry(topology=TRIANGLES)
        # Front face
        self.face(g, Vec3(-w / 2 + sx, -h / 2 + sy, -d / 2 + sz),
                  Vec3(w + sx, -h / 2 + sy, -d / 2 + sz),
                  Vec3(w + sx, h + sy, -d / 2 + sz),
                  Vec3(-w / 2 + sx, h + sy, -d / 2 + sz))

        # Back face
        self.face(g, Vec3(-w / 2 + sx, -h / 2 + sy, d + sz),
                  Vec3(w + sx, -h / 2 + sy, d + sz),
                  Vec3(w + sx, h + sy, d + sz),
                  Vec3(-w / 2 + sx, h + sy, d + sz))

        # Left face
        self.face(g, Vec3(-w / 2 + sx, -h / 2 + sy, -d / 2 + sz),
                  Vec3(-w / 2 + sx, -h / 2 + sy, d + sz),
                  Vec3(-w / 2 + sx, h + sy, d + sz),
                  Vec3(-w / 2 + sx, h + sy, -d / 2 + sz))

        # Right face
        self.face(g, Vec3(w + sx, -h / 2 + sy, -d / 2 + sz),
                  Vec3(w + sx, -h / 2 + sy, d + sz),
                  Vec3(w + sx, h + sy, d + sz),
                  Vec3(w + sx, h + sy, -d / 2 + sz))

        # Top face
        self.face(g, Vec3(-w / 2 + sx, -h / 2 + sy, -d / 2 + sz),
                  Vec3(w + sx, -h / 2 + sy, -d / 2 + sz),
                  Vec3(w + sx, -h / 2 + sy, d + sz),
                  Vec3(-w / 2 + sx, -h / 2 + sy, d + sz))

        # Bottom face
        self.face(g, Vec3(-w / 2 + sx, h + sy, -d / 2 + sz),
                  Vec3(w + sx, h + sy, -d / 2 + sz),
                  Vec3(w + sx, h + sy, d + sz),
                  Vec3(-w / 2 + sx, h + sy, d + sz))
        return g

    # One quad (two triangles) with a flat normal that points away from the cube's center
    def face(self, g, a, b, c, d):
        center = Vec3(self.shift_x + self.w / 4, self.shift_y + self.h / 4, self.shift_z + self.d / 4)
        n = (b - a).cross(c - a)
        n.normalize()
        if n.dot(a - center) < 0:
            n = n * -1
        g.normal(n.x, n.y, n.z)
        start = g.vertex_count()
        g.vertex(a.x, a.y, a.z)
        g.vertex(b.x, b.y, b.z)
        g.vertex(c.x, c.y, c.z)
        g.vertex(d.x, d.y, d.z)
        for i in (0, 1, 2, 0, 2, 3):
            g.index(start + i)

    # Main cube drawing method
    def draw_cube(self):
        draw_geometry(self.geometry)

        # Add some rotation to each box for pizazz.
        rotate_y(radians(1))
        rotate_x(radians(1))
        rotate_z(radians(1))


run()
