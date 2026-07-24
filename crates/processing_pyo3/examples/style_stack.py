# Style Stack
#
# A grid of spinners. Each cell sets up its own transform and color inside
# push()/pop(), so nothing leaks to its neighbours.
from mewnala import *
from math import sin

GRID = 6
t = 0.0


def setup():
    size(600, 600)


def draw():
    global t
    background(16, 16, 20)

    cell = width / GRID
    for gy in range(GRID):
        for gx in range(GRID):
            spin = t + (gx + gy) * 0.4
            pulse = 0.5 + 0.5 * sin(spin)

            push()
            translate((gx + 0.5) * cell, (gy + 0.5) * cell)
            rotate(spin)

            no_stroke()
            fill(235, 90 + 130 * pulse, 60)
            rect_mode(CENTER)
            rect(0, 0, cell * (0.3 + 0.15 * pulse), cell * 0.3)

            no_fill()
            stroke(255, 255, 255, 40)
            circle(0, 0, cell * 0.72)
            pop()

    t += 0.02


run()
