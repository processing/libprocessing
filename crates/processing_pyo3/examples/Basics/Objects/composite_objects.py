# Composite Objects
#
# An object can include several other objects. Creating such composite objects
# is a good way to use the principles of modularity and build higher levels of
# abstraction within a program.
from mewnala import *

er1 = None
er2 = None


def setup():
    global er1, er2
    size(640, 360)
    er1 = EggRing(width * 0.45, height * 0.5, 2, 120)
    er2 = EggRing(width * 0.65, height * 0.8, 10, 180)


def draw():
    background(0.0)
    er1.transmit()
    er2.transmit()


class Egg:
    def __init__(self, xpos, ypos, r, s):
        self.x = xpos
        self.y = ypos
        self.tilt = 0  # Left and right angle offset
        self.angle = 0.0  # Used to define the tilt
        self.scalar = s / 100.0  # Height of the egg
        self.range = r

    def wobble(self):
        self.tilt = cos(self.angle) / self.range
        self.angle += 0.1

    def display(self):
        no_stroke()
        fill(1.0)
        push_matrix()
        translate(self.x, self.y)
        rotate(self.tilt)
        scale(self.scalar)
        begin_shape()
        vertex(0, -100)
        bezier_vertex(25, -100, 40, -65, 40, -40)
        bezier_vertex(40, -15, 25, 0, 0, 0)
        bezier_vertex(-25, 0, -40, -15, -40, -40)
        bezier_vertex(-40, -65, -25, -100, 0, -100)
        end_shape()
        pop_matrix()


class EggRing:
    def __init__(self, x, y, t, sp):
        self.ovoid = Egg(x, y, t, sp)
        self.circle = Ring()
        self.circle.start(x, y - sp / 2)

    def transmit(self):
        self.ovoid.wobble()
        self.ovoid.display()
        self.circle.grow()
        self.circle.display()
        if not self.circle.on:
            self.circle.on = True


class Ring:
    def __init__(self):
        self.x = 0.0
        self.y = 0.0
        self.diameter = 0.0  # Diameter of the ring
        self.on = False  # Turns the display on and off

    def start(self, xpos, ypos):
        self.x = xpos
        self.y = ypos
        self.on = True
        self.diameter = 1

    def grow(self):
        if self.on:
            self.diameter += 0.5
            if self.diameter > width * 2:
                self.diameter = 0.0

    def display(self):
        if self.on:
            no_fill()
            stroke_weight(4)
            stroke(0.61, 0.6)
            ellipse(self.x, self.y, self.diameter, self.diameter)


run()
