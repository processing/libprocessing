# PolygonPShape
#
# Using a retained geometry to display a custom polygon.
from mewnala import *

# The retained shape
star = None


def setup():
    global star
    size(640, 360)

    # First create the shape
    star = create_geometry(topology=TRIANGLES)
    # You can set fill and stroke
    star.color(0.4, 0.4, 0.4, 1.0)
    stroke(1.0)  # GAP: Geometry has no stroke; the outline is not drawn
    stroke_weight(2)  # GAP: Geometry has no stroke_weight
    # Here, we are hardcoding a series of vertices;
    # the first one is the center, so the star can be built as a triangle fan
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


def draw():
    background(0.2)
    # We can use translate to move the shape
    translate(mouse_x, mouse_y)
    # Display the shape
    draw_geometry(star)


run()
