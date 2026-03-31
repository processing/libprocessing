from mewnala import *
from math import sin, cos

t = 0.0

def setup():
    size(800, 400)

def draw():
    global t
    background(10, 10, 15)

    no_fill()

    stroke_weight(1.5)
    for i in range(20):
        y_base = 20 + i * 15
        phase = t + i * 0.3
        amp = 30 + sin(i * 0.5) * 20

        v = 0.3 + (i / 20.0) * 0.5
        stroke(v * 0.6 * 255, v * 255, min(v * 1.2, 1.0) * 255)

        bezier(
            0, y_base,
            250, y_base + amp * sin(phase),
            550, y_base - amp * cos(phase * 1.3),
            800, y_base,
        )

    stroke_weight(0.8)
    for i in range(8):
        y_base = 50 + i * 40
        phase = t * 0.7 + i * 0.5

        stroke(255, 153, 51, 102)

        curve(
            -50, y_base + 40 * sin(phase),
            200, y_base + 30 * cos(phase * 1.2),
            600, y_base - 30 * sin(phase * 0.8),
            850, y_base + 20 * cos(phase),
        )

    stroke_weight(0.3)
    stroke(255, 255, 255, 20)
    for i in range(20):
        y = 20.0 * i
        line(0, y, 800, y)

    t += 0.012

run()
