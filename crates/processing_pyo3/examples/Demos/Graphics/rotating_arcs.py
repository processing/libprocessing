# Geometry
# by Marius Watz.
#
# Using sin/cos, blends colors, and draws a series of
# rotating arcs on the screen.
from mewnala import *

COUNT = 150

pt = []  # rotx, roty, deg, rad, w, speed
style = []  # color, render style


def setup():
    global pt, style
    size(1024, 768)
    mode_3d()
    background(1.0)
    # random_seed(100)  # use this to get the same result each time

    pt = [0.0] * (6 * COUNT)
    style = [0] * (2 * COUNT)

    # Set up arc shapes
    index = 0
    for i in range(COUNT):
        pt[index] = random(TAU)  # Random X axis rotation
        index += 1
        pt[index] = random(TAU)  # Random Y axis rotation
        index += 1

        pt[index] = random(60, 80)  # Short to quarter-circle arcs
        if random(100) > 90:
            pt[index] = floor(random(8, 27)) * 10
        index += 1

        pt[index] = int(random(2, 50) * 5)  # Radius. Space them out nicely
        index += 1

        pt[index] = random(4, 32)  # Width of band
        if random(100) > 90:
            pt[index] = random(40, 60)  # Width of band
        index += 1

        pt[index] = radians(random(5, 30)) / 5  # Speed of rotation
        index += 1

        prob = random(100)
        if prob < 50:
            style[i * 2] = color_blended(random(1), 0.78, 1.0, 0.0, 0.2, 0.47, 0.0, 0.82)
        elif prob < 90:
            style[i * 2] = color_blended(random(1), 1.0, 0.39, 0.0, 1.0, 1.0, 0.0, 0.82)
        else:
            style[i * 2] = color(1.0, 1.0, 1.0, 0.86)

        style[i * 2 + 1] = floor(random(100)) % 3


def draw():
    background(0.0)

    rotate_x(PI / 6)
    rotate_y(PI / 6)

    index = 0
    for i in range(COUNT):
        push_matrix()
        rotate_x(pt[index])
        index += 1
        rotate_y(pt[index])
        index += 1

        if style[i * 2 + 1] == 0:
            stroke(style[i * 2])
            no_fill()
            stroke_weight(1)
            arc_line(0, 0, pt[index], pt[index + 1], pt[index + 2])
        elif style[i * 2 + 1] == 1:
            fill(style[i * 2])
            no_stroke()
            arc_line_bars(0, 0, pt[index], pt[index + 1], pt[index + 2])
        else:
            fill(style[i * 2])
            no_stroke()
            solid_arc(0, 0, pt[index], pt[index + 1], pt[index + 2])
        index += 3

        # increase rotation
        pt[index - 5] += pt[index] / 10
        pt[index - 4] += pt[index] / 20
        index += 1

        pop_matrix()


# Get blend of two colors
def color_blended(fract, r, g, b, r2, g2, b2, a):
    return color(r + (r2 - r) * fract, g + (g2 - g) * fract, b + (b2 - b) * fract, a)


# Draw arc line
def arc_line(x, y, degrees, radius, w):
    line_count = floor(w / 2)

    for j in range(line_count):
        begin_shape()
        for i in range(int(degrees)):  # one step for each degree
            angle = radians(i)
            vertex(x + cos(angle) * radius, y + sin(angle) * radius)
        end_shape()
        radius += 2


# Draw arc line with bars
def arc_line_bars(x, y, degrees, radius, w):
    begin_shape(QUADS)
    i = 0
    while i < degrees / 4:  # degrees, but in steps of 4
        angle = radians(i)
        vertex(x + cos(angle) * radius, y + sin(angle) * radius)
        vertex(x + cos(angle) * (radius + w), y + sin(angle) * (radius + w))

        angle = radians(i + 2)
        vertex(x + cos(angle) * (radius + w), y + sin(angle) * (radius + w))
        vertex(x + cos(angle) * radius, y + sin(angle) * radius)
        i += 4
    end_shape()


# Draw solid arc (named arc() in the original; renamed so it doesn't shadow arc())
def solid_arc(x, y, degrees, radius, w):
    begin_shape(QUAD_STRIP)
    for i in range(int(degrees)):
        angle = radians(i)
        vertex(x + cos(angle) * radius, y + sin(angle) * radius)
        vertex(x + cos(angle) * (radius + w), y + sin(angle) * (radius + w))
    end_shape()


run()
