from mewnala import *
from math import sin, cos, pi

second_window = None
N = 7

def points(w, h, t):
    pts = []
    for i in range(N):
        angle = t * (0.3 + i * 0.05) + i * (2 * pi / N)
        radius = min(w, h) * (0.18 + 0.12 * sin(t * 0.7 + i))
        x = w / 2 + cos(angle) * radius
        y = h / 2 + sin(angle * 1.3) * radius
        pts.append((x, y))
    return pts


def setup():
    global second_window
    size(480, 480)
    second_window = create_window(360, 360, "Second window")


def draw():
    t = frame_count * 0.03

    background(20, 16, 28)
    no_stroke()
    for i, (x, y) in enumerate(points(width, height, t)):
        fill(255, 120 + i * 15, 80)
        circle(x, y, 26)

    g = second_window
    g.background(10, 12, 24)
    g.stroke(120, 180, 255)
    g.stroke_weight(1.5)
    pts = points(g.width, g.height, t)
    link = (g.width * 0.35) ** 2
    for i in range(N):
        for j in range(i + 1, N):
            (x1, y1), (x2, y2) = pts[i], pts[j]
            if (x1 - x2) ** 2 + (y1 - y2) ** 2 < link:
                g.line(x1, y1, x2, y2)
    g.no_stroke()
    g.fill(220, 235, 255)
    for (x, y) in pts:
        g.circle(x, y, 6)


run()
