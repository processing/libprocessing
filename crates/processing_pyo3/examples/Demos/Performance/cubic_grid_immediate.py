# Cubic Grid
# by Ira Greenberg.
#
# 3D translucent colored grid uses nested push_matrix()
# and pop_matrix() functions.
from mewnala import *

box_size = 20
margin = box_size * 2
depth = 400
box_fill = None

fcount = 0
lastm = 0
frate = 0
fint = 3


def setup():
    size(640, 360)
    mode_3d()
    frame_rate(60)
    no_stroke()


def draw():
    global box_fill, fcount, lastm, frate
    background(1.0)

    # Center and spin grid
    push_matrix()
    translate(0, 0, -depth)
    rotate_y(frame_count * 0.01)
    rotate_x(frame_count * 0.01)

    # Build grid using multiple translations
    i = -depth / 2 + margin
    while i <= depth / 2 - margin:
        j = -height + margin
        while j <= height - margin:
            k = -width + margin
            while k <= width - margin:
                # Base fill color on counter values, abs function
                # ensures values stay within legal range
                box_fill = color(constrain(abs(i), 0, 255) / 255, constrain(abs(j), 0, 255) / 255, constrain(abs(k), 0, 255) / 255, 0.2)
                push_matrix()
                translate(k, j, i)
                fill(box_fill)
                box(box_size, box_size, box_size)
                pop_matrix()
                k += box_size
            j += box_size
        i += box_size
    pop_matrix()

    fcount += 1
    m = millis()
    if m - lastm > 1000 * fint:
        frate = fcount / fint
        fcount = 0
        lastm = m
        print("fps: " + str(frate))
    window_title("fps: " + str(frate))


run()
