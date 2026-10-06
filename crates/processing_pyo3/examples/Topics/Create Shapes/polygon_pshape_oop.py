# PolygonPShapeOOP
#
# Wrapping a retained geometry inside a custom class
from mewnala import *

# A Star object
s1 = None
s2 = None


def setup():
    global s1, s2
    size(640, 360)

    # Make a new Star
    s1 = Star()
    s2 = Star()


def draw():
    background(0.2)

    s1.display()  # Display the first star
    s1.move()  # Move the first star

    s2.display()  # Display the second star
    s2.move()  # Move the second star


# A class to describe a Star shape
class Star:
    def __init__(self):
        # The location where we will draw the shape
        self.x = random(100, width - 100)
        self.y = random(100, height - 100)
        self.speed = random(0.5, 3)
        # First create the shape
        self.s = create_geometry(topology=TRIANGLES)
        # You can set fill and stroke
        self.s.color(1.0, 1.0, 1.0, 0.8)
        # Here, we are hardcoding a series of vertices;
        # the first one is the center, so the star can be built as a triangle fan
        self.s.vertex(0, 0, 0)
        self.s.vertex(0, -50, 0)
        self.s.vertex(14, -20, 0)
        self.s.vertex(47, -15, 0)
        self.s.vertex(23, 7, 0)
        self.s.vertex(29, 40, 0)
        self.s.vertex(0, 25, 0)
        self.s.vertex(-29, 40, 0)
        self.s.vertex(-23, 7, 0)
        self.s.vertex(-47, -15, 0)
        self.s.vertex(-14, -20, 0)
        # One triangle from the center to each edge of the outline
        for i in range(1, 11):
            self.s.index(0)
            self.s.index(i)
            self.s.index(i % 10 + 1)

    def move(self):
        # Demonstrating some simple motion
        self.x += self.speed
        if self.x > width + 100:
            self.x = -100

    def display(self):
        # Locating and drawing the shape
        push_matrix()
        translate(self.x, self.y)
        no_stroke()
        draw_geometry(self.s)
        pop_matrix()


run()
