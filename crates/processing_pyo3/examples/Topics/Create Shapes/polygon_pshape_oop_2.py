# PolygonPShapeOOP2
#
# Wrapping a retained geometry inside a custom class
# and demonstrating how we can have a multiple objects each
# using the same geometry.
from mewnala import *

# A list of objects
polygons = []


def setup():
    size(640, 360)

    # Make a retained star
    star = create_geometry(topology=TRIANGLES)
    star.color(0.0, 0.0, 0.0, 0.5)
    # The first vertex is the center, so the star can be built as a triangle fan
    star.vertex(0, 0, 0)
    star.vertex(0, -50, 0)
    star.vertex(14, -20, 0)
    star.vertex(47, -15, 0)
    star.vertex(23, 7, 0)
    star.vertex(29, 40, 0)
    star.vertex(0, 25, 0)
    star.vertex(-29, 40, 0)
    star.vertex(-23, 7, 0)
    star.vertex(-47, -15, 0)
    star.vertex(-14, -20, 0)
    # One triangle from the center to each edge of the outline
    for i in range(1, 11):
        star.index(0)
        star.index(i)
        star.index(i % 10 + 1)

    # Add a bunch of objects to the list
    # Pass in reference to the geometry
    # We could make polygons with different geometries
    for i in range(25):
        polygons.append(Polygon(star))


def draw():
    background(1.0)

    # Display and move them all
    for poly in polygons:
        poly.display()
        poly.move()


# A class to describe a Polygon (with a retained geometry)
class Polygon:
    def __init__(self, s):
        # The location where we will draw the shape
        self.x = random(width)
        self.y = random(-500, -100)
        # The geometry
        self.s = s
        # Variable for simple motion
        self.speed = random(2, 6)

    # Simple motion
    def move(self):
        self.y += self.speed
        if self.y > height + 100:
            self.y = -100

    # Draw the object
    def display(self):
        push_matrix()
        translate(self.x, self.y)
        draw_geometry(self.s)
        pop_matrix()


run()
