# I Like Icosahedra
# by Ira Greenberg.
#
# This example plots icosahedra. The Icosahdron is a regular
# polyhedron composed of twenty equalateral triangles.
from mewnala import *

ico1 = None
ico2 = None
ico3 = None
light = None


def setup():
    global ico1, ico2, ico3, light
    size(640, 360)
    mode_3d()
    light = directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)
    ico1 = Icosahedron(75)
    ico2 = Icosahedron(75)
    ico3 = Icosahedron(75)


def draw():
    background(0.0)

    push_matrix()
    translate(-width / 3.5, 0)
    rotate_x(frame_count * PI / 185)
    rotate_y(frame_count * PI / -200)
    stroke(0.67, 0.0, 0.0)
    no_fill()
    ico1.create()  # GAP: stroke()/no_fill() wireframe outline on draw_geometry
    pop_matrix()

    push_matrix()
    rotate_x(frame_count * PI / 200)
    rotate_y(frame_count * PI / 300)
    stroke(0.59, 0.0, 0.71)
    fill(0.67, 0.67, 0.0)
    ico2.create()  # GAP: stroke() outline on draw_geometry
    pop_matrix()

    push_matrix()
    translate(width / 3.5, 0)
    rotate_x(frame_count * PI / -200)
    rotate_y(frame_count * PI / 200)
    no_stroke()
    fill(0.0, 0.0, 0.73)
    ico3.create()
    pop_matrix()


class Dimension3D:
    def __init__(self, w, h, d):
        self.w = w
        self.h = h
        self.d = d


class Shape3D:
    def __init__(self, x=0.0, y=0.0, z=0.0, w=0.0, h=0.0, d=0.0):
        self.x = x
        self.y = y
        self.z = z
        self.w = w
        self.h = h
        self.d = d

    def set_loc(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z

    # override if you need these
    def rot_x(self, theta):
        pass

    def rot_y(self, theta):
        pass

    def rot_z(self, theta):
        pass

    # must be implemented in subclasses
    def init(self):
        raise NotImplementedError

    def create(self):
        raise NotImplementedError


class Icosahedron(Shape3D):
    # constructor
    def __init__(self, radius, v=None):
        if v is None:
            super().__init__()
        else:
            super().__init__(v.x, v.y, v.z)
        self.top_point = None
        self.top_pent = []
        self.bottom_point = None
        self.bottom_pent = []
        self.angle = 0
        self.radius = radius
        self.tri_dist = 0
        self.tri_ht = 0
        self.a = 0
        self.b = 0
        self.c = 0
        self.geometry = None
        self.init()

    # calculate geometry
    def init(self):
        self.c = dist(cos(0) * self.radius, sin(0) * self.radius, cos(radians(72)) * self.radius, sin(radians(72)) * self.radius)
        self.b = self.radius
        self.a = sqrt((self.c * self.c) - (self.b * self.b))

        self.tri_ht = sqrt((self.c * self.c) - ((self.c / 2) * (self.c / 2)))

        for i in range(5):
            self.top_pent.append(Vec3(cos(self.angle) * self.radius, sin(self.angle) * self.radius, self.tri_ht / 2.0))
            self.angle += radians(72)
        self.top_point = Vec3(0, 0, self.tri_ht / 2.0 + self.a)
        self.angle = 72.0 / 2.0
        for i in range(5):
            self.bottom_pent.append(Vec3(cos(self.angle) * self.radius, sin(self.angle) * self.radius, -self.tri_ht / 2.0))
            self.angle += radians(72)
        self.bottom_point = Vec3(0, 0, -(self.tri_ht / 2.0 + self.a))
        self.build()

    # the twenty faces become one triangle mesh, built once
    def build(self):
        self.geometry = create_geometry(topology=TRIANGLES)
        top = self.top_pent
        bottom = self.bottom_pent
        for i in range(5):
            # icosahedron top (the last triangle wraps around to the first point)
            self.face(top[i], self.top_point, top[(i + 1) % 5])
            # icosahedron bottom
            self.face(bottom[i], self.bottom_point, bottom[(i + 1) % 5])

        # icosahedron body
        for i in range(5):
            self.face(top[i], bottom[(i + 1) % 5], bottom[(i + 2) % 5])
            self.face(bottom[(i + 2) % 5], top[i], top[(i + 1) % 5])

    # one flat-shaded triangle; lit geometry needs a normal, pointing outwards
    def face(self, a, b, c):
        n = (b - a).cross(c - a)
        n.normalize()
        if n.dot(a) < 0:
            n = n * -1
        self.geometry.normal(n.x, n.y, n.z)
        self.geometry.vertex(a.x, a.y, a.z)
        self.geometry.vertex(b.x, b.y, b.z)
        self.geometry.vertex(c.x, c.y, c.z)

    # draws icosahedron
    def create(self):
        push_matrix()
        translate(self.x, self.y, self.z)
        draw_geometry(self.geometry)
        pop_matrix()

    # overrided methods fom Shape3D
    def rot_z(self, theta):
        # top point
        tx = cos(theta) * self.top_point.x + sin(theta) * self.top_point.y
        ty = sin(theta) * self.top_point.x - cos(theta) * self.top_point.y
        self.top_point.x = tx
        self.top_point.y = ty

        # bottom point
        tx = cos(theta) * self.bottom_point.x + sin(theta) * self.bottom_point.y
        ty = sin(theta) * self.bottom_point.x - cos(theta) * self.bottom_point.y
        self.bottom_point.x = tx
        self.bottom_point.y = ty

        # top and bottom pentagons
        for i in range(5):
            tx = cos(theta) * self.top_pent[i].x + sin(theta) * self.top_pent[i].y
            ty = sin(theta) * self.top_pent[i].x - cos(theta) * self.top_pent[i].y
            self.top_pent[i].x = tx
            self.top_pent[i].y = ty

            tx = cos(theta) * self.bottom_pent[i].x + sin(theta) * self.bottom_pent[i].y
            ty = sin(theta) * self.bottom_pent[i].x - cos(theta) * self.bottom_pent[i].y
            self.bottom_pent[i].x = tx
            self.bottom_pent[i].y = ty
        self.build()

    def rot_x(self, theta):
        pass

    def rot_y(self, theta):
        pass


run()
