# Inheritance
#
# A class can be defined using another class as a foundation. In object-oriented
# programming terminology, one class can inherit fields and methods from another.
# An object that inherits from another is called a subclass, and the object it
# inherits from is called a superclass. A subclass extends the superclass.
from mewnala import *

spots = None
arm = None


def setup():
    global spots, arm
    size(640, 360)
    arm = SpinArm(width / 2, height / 2, 0.01)
    spots = SpinSpots(width / 2, height / 2, -0.02, 90.0)


def draw():
    background(0.8)
    arm.update()
    arm.display()
    spots.update()
    spots.display()


class Spin:
    def __init__(self, xpos, ypos, s):
        self.x = xpos
        self.y = ypos
        self.speed = s
        self.angle = 0.0

    def update(self):
        self.angle += self.speed


class SpinArm(Spin):
    def __init__(self, x, y, s):
        super().__init__(x, y, s)

    def display(self):
        stroke_weight(1)
        stroke(0.0)
        push_matrix()
        translate(self.x, self.y)
        self.angle += self.speed
        rotate(self.angle)
        line(0, 0, 165, 0)
        pop_matrix()


class SpinSpots(Spin):
    def __init__(self, x, y, s, d):
        super().__init__(x, y, s)
        self.dim = d

    def display(self):
        no_stroke()
        push_matrix()
        translate(self.x, self.y)
        self.angle += self.speed
        rotate(self.angle)
        ellipse(-self.dim / 2, 0, self.dim, self.dim)
        ellipse(self.dim / 2, 0, self.dim, self.dim)
        pop_matrix()


run()
