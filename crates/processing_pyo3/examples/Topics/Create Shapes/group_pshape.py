# GroupPShape
#
# How to group multiple PShapes into one PShape
from mewnala import *

# A PShape that will group PShapes
group = None


def setup():
    global group
    size(640, 360)

    # Create the shape as a group
    group = create_shape(GROUP)  # GAP: create_shape(GROUP) retained shape group

    # Make a polygon PShape
    star = create_shape()  # GAP: create_shape() retained shape with its own style
    star.begin_shape()  # GAP: Shape.begin_shape
    star.no_fill()  # GAP: Shape.no_fill
    star.stroke(1.0)  # GAP: Shape.stroke
    star.vertex(0, -50)  # GAP: Shape.vertex
    star.vertex(14, -20)
    star.vertex(47, -15)
    star.vertex(23, 7)
    star.vertex(29, 40)
    star.vertex(0, 25)
    star.vertex(-29, 40)
    star.vertex(-23, 7)
    star.vertex(-47, -15)
    star.vertex(-14, -20)
    star.end_shape(CLOSE)  # GAP: Shape.end_shape

    # Make a path PShape
    path = create_shape()  # GAP: create_shape()
    path.begin_shape()
    path.no_fill()
    path.stroke(1.0)
    a = -PI
    while a < 0:
        r = random(60, 70)
        path.vertex(r * cos(a), r * sin(a))
        a += 0.1
    path.end_shape()

    # Make a primitive (Rectangle) PShape
    rectangle = create_shape(RECT, -10, -10, 20, 20)  # GAP: create_shape(RECT, x, y, w, h) primitive shape
    rectangle.set_fill(False)  # GAP: Shape.set_fill(False)
    rectangle.set_stroke(color(1.0))  # GAP: Shape.set_stroke(color)

    # Add them all to the group
    group.add_child(star)  # GAP: Shape.add_child
    group.add_child(path)
    group.add_child(rectangle)


def draw():
    # We can access them individually via the group PShape
    rectangle = group.get_child(2)  # GAP: Shape.get_child(i)
    # Shapes can be rotated
    rectangle.rotate(0.1)  # GAP: Shape.rotate (accumulating shape-local transform)

    background(0.2)
    # Display the group PShape
    translate(mouse_x, mouse_y)
    shape(group)  # GAP: shape(s) draws a retained shape


run()
