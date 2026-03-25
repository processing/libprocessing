from mewnala import *
from math import pi, sin, cos

t = 0.0

def setup():
    size(600, 600)

def draw():
    global t
    background(242, 237, 230)

    cx = 300.0
    cy = 300.0

    no_fill()
    stroke_weight(2)

    for i in range(12):
        r = 40 + i * 18
        offset = t + i * 0.15
        v = 0.2 + (i / 12.0) * 0.4
        stroke(v * 255, v * 0.8 * 255, v * 0.6 * 255)
        arc(cx, cy, r * 2, r * 2, offset, offset + pi * 1.2, OPEN)

    no_stroke()
    for i in range(12):
        r = 40 + i * 18
        angle = t + i * 0.15
        x = cx + r * cos(angle)
        y = cy + r * sin(angle)
        fill(217, (0.3 + i / 24.0) * 255, 51)
        ellipse(x, y, 6, 6)

    fill(25, 25, 25, 38)
    for i in range(6):
        angle = t * 0.5 + i * pi / 3.0
        d = 250.0
        px = cx + d * cos(angle)
        py = cy + d * sin(angle)
        s = 20.0
        triangle(px, py - s, px - s * 0.866, py + s * 0.5, px + s * 0.866, py + s * 0.5)

    d = 15 + 5 * sin(t * 3)
    fill(230, 102, 25, 102)
    quad(cx, cy - d, cx + d, cy, cx, cy + d, cx - d, cy)

    t += 0.01

run()
