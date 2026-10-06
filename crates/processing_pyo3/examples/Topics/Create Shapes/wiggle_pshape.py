# WigglePShape
#
# How to move the individual vertices of a retained geometry
from mewnala import *

# A "Wiggler" object
w = None


def setup():
    global w
    size(640, 360)
    w = Wiggler()


def draw():
    background(1.0)
    w.display()
    w.wiggle()


# An object that wraps the geometry
class Wiggler:
    def __init__(self):
        # Its location
        self.x = width / 2
        self.y = height / 2

        # For 2D Perlin noise
        self.yoff = 0.0

        # We are using a list to keep a duplicate copy
        # of vertices original locations.
        # The "original" locations of the vertices make up a circle
        self.original = []
        a = 0.0
        while a < TWO_PI:
            v = Vec2.from_angle(a)
            v.mult(100)
            self.original.append(v)
            a += 0.2

        # Now make the geometry with those vertices
        self.s = create_geometry(topology=TRIANGLES)
        self.s.color(0.5, 0.5, 0.5, 1.0)
        stroke(0.0)  # GAP: Geometry has no stroke; the outline is not drawn
        stroke_weight(2)  # GAP: Geometry has no stroke_weight
        for v in self.original:
            self.s.vertex(v.x, v.y, 0)
        # The circle is convex, so a triangle fan from the first vertex fills it
        for i in range(1, len(self.original) - 1):
            self.s.index(0)
            self.s.index(i)
            self.s.index(i + 1)

    def wiggle(self):
        xoff = 0.0
        # Apply an offset to each vertex
        for i in range(self.s.vertex_count()):
            # Calculate a new vertex location based on noise around "original" location
            pos = self.original[i]
            a = TWO_PI * noise(xoff, self.yoff)
            r = Vec2.from_angle(a)
            r.mult(4)
            r.add(pos)
            # Set the location of each vertex to the new one
            self.s.set_vertex(i, r.x, r.y, 0)
            # increment perlin noise x value
            xoff += 0.5
        # Increment perlin noise y value
        self.yoff += 0.02

    def display(self):
        push_matrix()
        translate(self.x, self.y)
        draw_geometry(self.s)
        pop_matrix()


run()
