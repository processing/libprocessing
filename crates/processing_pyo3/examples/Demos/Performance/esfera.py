# Esfera
# by David Pena.
#
# Distribucion aleatoria uniforme sobre la superficie de una esfera.
# The hairs are kept in one LINES geometry whose vertices are moved every frame.
from mewnala import *

cuantos = 16000
lista = []
radio = 200
rx = 0
ry = 0
pelos = None


def setup():
    global radio, pelos
    size(1024, 768)
    mode_3d()

    radio = height / 3.5

    # Two vertices per hair: black at the root, translucent gray at the tip
    pelos = create_geometry(topology=LINES)
    for i in range(cuantos):
        lista.append(Pelo(i))
        pelos.color(0.0, 0.0, 0.0, 1.0)
        pelos.vertex(0, 0, 0)
        pelos.color(0.78, 0.78, 0.78, 0.59)
        pelos.vertex(0, 0, 0)
        pelos.index(2 * i)
        pelos.index(2 * i + 1)
    noise_detail(3)


def draw():
    global rx, ry
    background(0.0)

    rxp = (mouse_x - (width / 2)) * 0.005
    ryp = (mouse_y - (height / 2)) * 0.005
    rx = rx * 0.9 + rxp * 0.1
    ry = ry * 0.9 + ryp * 0.1

    rotate_y(rx)
    rotate_x(ry)
    fill(0.0)
    no_stroke()
    sphere(radio)

    for i in range(len(lista)):
        lista[i].dibujar()

    stroke_weight(1)
    draw_geometry(pelos)


class Pelo:
    def __init__(self, index):
        self.index = index
        self.z = random(-radio, radio)
        self.phi = random(TWO_PI)
        self.largo = random(1.15, 1.2)
        self.theta = asin(self.z / radio)

    def dibujar(self):
        off = (noise(millis() * 0.0005, sin(self.phi)) - 0.5) * 0.3
        offb = (noise(millis() * 0.0007, sin(self.z) * 0.01) - 0.5) * 0.3

        thetaff = self.theta + off
        phff = self.phi + offb
        x = radio * cos(self.theta) * cos(self.phi)
        y = radio * cos(self.theta) * sin(self.phi)
        z = radio * sin(self.theta)

        xo = radio * cos(thetaff) * cos(phff)
        yo = radio * cos(thetaff) * sin(phff)
        zo = radio * sin(thetaff)

        xb = xo * self.largo
        yb = yo * self.largo
        zb = zo * self.largo

        # Move this hair's two vertices in the shared geometry
        pelos.set_vertex(2 * self.index, x, y, z)
        pelos.set_vertex(2 * self.index + 1, xb, yb, zb)


run()
