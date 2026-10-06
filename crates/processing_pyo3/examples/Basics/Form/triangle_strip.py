# Triangle Strip
# by Ira Greenberg.
#
# Generate a closed ring using the vertex() function and
# begin_shape(TRIANGLE_STRIP) mode. The outside_radius and inside_radius
# variables control ring's radii respectively.
from mewnala import *

x = 0
y = 0
outside_radius = 150
inside_radius = 100


def setup():
    global x, y
    size(640, 360)
    background(0.8)
    x = width // 2
    y = height // 2


def draw():
    background(0.8)

    num_points = int(remap(mouse_x, 0, width, 6, 60))
    angle = 0
    angle_step = 180.0 / num_points

    begin_shape(TRIANGLE_STRIP)
    for i in range(num_points + 1):
        px = x + cos(radians(angle)) * outside_radius
        py = y + sin(radians(angle)) * outside_radius
        angle += angle_step
        vertex(px, py)
        px = x + cos(radians(angle)) * inside_radius
        py = y + sin(radians(angle)) * inside_radius
        vertex(px, py)
        angle += angle_step
    end_shape()


run()
