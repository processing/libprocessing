from mewnala import *
from math import pi, sin, cos

t = 0.0

def setup():
    size(600, 600)

def draw():
    global t
    background(235, 230, 222)

    fill(51, 76, 127, 64)
    stroke_weight(1.5)
    stroke(38, 51, 89)

    n = 8
    begin_shape(POLYGON)
    for i in range(n + 3):
        idx = (i + n - 1) % n
        angle = idx * 2 * pi / n
        r = 100 + 30 * sin(t * 2 + idx * 0.8)
        curve_vertex(170 + r * cos(angle), 200 + r * sin(angle))
    end_shape(CLOSE)

    fill(38, 127, 76, 51)
    stroke_weight(1)
    stroke(25, 89, 51)

    pcx = 430
    pcy = 200
    s = 70 + 10 * sin(t * 1.5)

    begin_shape(POLYGON)
    vertex(pcx, pcy - s)
    bezier_vertex(pcx + s, pcy - s, pcx + s, pcy + s, pcx, pcy + s)
    bezier_vertex(pcx - s, pcy + s, pcx - s, pcy - s, pcx, pcy - s)
    end_shape(CLOSE)

    fill(204, 89, 25, 38)
    no_stroke()

    begin_shape(TRIANGLE_FAN)
    vertex(170, 450)
    for i in range(17):
        angle = i * 2 * pi / 16
        r = 60 + 20 * sin(t * 3 + angle * 2)
        vertex(170 + r * cos(angle), 450 + r * sin(angle))
    end_shape(CLOSE)

    fill(127, 51, 153, 51)
    stroke_weight(0.5)
    stroke(76, 25, 102, 102)

    begin_shape(TRIANGLE_STRIP)
    for i in range(16):
        x = 320 + i * 17
        wave = 30 * sin(t * 2 + i * 0.4)
        vertex(x, 420 + wave)
        vertex(x, 480 + wave)
    end_shape()

    t += 0.02

run()
