# PolygonPShapeOOP3
#
# Wrapping a PShape inside a custom class
# and demonstrating how we can have a multiple objects each
# using the same PShape.
from mewnala import *

# A list of objects
polygons = []

# Three possible shapes
shapes = [None, None, None]


def setup():
    size(640, 360)

    shapes[0] = create_shape(ELLIPSE, 0, 0, 100, 100)  # GAP: create_shape(ELLIPSE, x, y, w, h) primitive shape
    shapes[0].set_fill(color(1.0, 0.5))  # GAP: Shape.set_fill(color)
    shapes[0].set_stroke(False)  # GAP: Shape.set_stroke(False)
    shapes[1] = create_shape(RECT, 0, 0, 100, 100)  # GAP: create_shape(RECT, x, y, w, h) primitive shape
    shapes[1].set_fill(color(1.0, 0.5))
    shapes[1].set_stroke(False)
    shapes[2] = create_shape()  # GAP: create_shape() retained shape with its own style
    shapes[2].begin_shape()  # GAP: Shape.begin_shape
    shapes[2].fill(0.0, 0.5)  # GAP: Shape.fill
    shapes[2].no_stroke()  # GAP: Shape.no_stroke
    shapes[2].vertex(0, -50)  # GAP: Shape.vertex
    shapes[2].vertex(14, -20)
    shapes[2].vertex(47, -15)
    shapes[2].vertex(23, 7)
    shapes[2].vertex(29, 40)
    shapes[2].vertex(0, 25)
    shapes[2].vertex(-29, 40)
    shapes[2].vertex(-23, 7)
    shapes[2].vertex(-47, -15)
    shapes[2].vertex(-14, -20)
    shapes[2].end_shape(CLOSE)  # GAP: Shape.end_shape

    for i in range(25):
        selection = int(random(len(shapes)))  # Pick a random index
        p = Polygon(shapes[selection])  # Use corresponding PShape to create Polygon
        polygons.append(p)


def draw():
    background(0.4)

    # Display and move them all
    for poly in polygons:
        poly.display()
        poly.move()


# A class to describe a Polygon (with a PShape)
class Polygon:
    def __init__(self, s):
        # The location where we will draw the shape
        self.x = random(width)
        self.y = random(-500, -100)
        # The PShape object
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
        shape(self.s)  # GAP: shape(s) draws a retained shape
        pop_matrix()


run()
