# Shape Vertices.
#
# How to iterate over the vertices of a shape.
# When loading an obj or SVG, get_vertex_count()
# will typically return 0 since all the vertices
# are in the child shapes.
#
# You should iterate through the children and then
# iterate through their vertices.
from mewnala import *

# The shape
uk = None


def setup():
    global uk
    size(640, 360)
    # Load the shape
    uk = load_shape("data/uk.svg")  # GAP: load_shape() (SVG) is missing


def draw():
    background(0.2)
    # Center where we will draw all the vertices
    translate(width / 2 - uk.width / 2, height / 2 - uk.height / 2)  # GAP: Shape.width / Shape.height

    # Iterate over the children
    children = uk.get_child_count()  # GAP: Shape.get_child_count()
    for i in range(children):
        child = uk.get_child(i)  # GAP: Shape.get_child(index)
        total = child.get_vertex_count()  # GAP: Shape.get_vertex_count()

        # Now we can actually get the vertices from each child
        for j in range(total):
            v = child.get_vertex(j)  # GAP: Shape.get_vertex(j) -> Vec2
            # Cycling brightness for each vertex
            stroke(((frame_count + (i + 1) * j) % 255) / 255)
            # Just a dot for each one
            point(v.x, v.y)


run()
