# Array Objects.
#
# Demonstrates the syntax for creating a list of custom objects.
from mewnala import *

unit = 40
count = 0
mods = []


def setup():
    global count
    size(640, 360)
    no_stroke()
    wide_count = width // unit
    high_count = height // unit
    count = wide_count * high_count

    for y in range(high_count):
        for x in range(wide_count):
            mods.append(Module(x * unit, y * unit, unit / 2, unit / 2, random(0.05, 0.8), unit))


def draw():
    background(0.0)
    for mod in mods:
        mod.update()
        mod.display()


class Module:
    def __init__(self, x_offset, y_offset, x, y, speed, unit):
        self.x_offset = x_offset
        self.y_offset = y_offset
        self.x = x
        self.y = y
        self.speed = speed
        self.unit = unit
        self.x_direction = 1
        self.y_direction = 1

    # Custom method for updating the variables
    def update(self):
        self.x = self.x + (self.speed * self.x_direction)
        if self.x >= self.unit or self.x <= 0:
            self.x_direction *= -1
            self.x = self.x + (1 * self.x_direction)
            self.y = self.y + (1 * self.y_direction)
        if self.y >= self.unit or self.y <= 0:
            self.y_direction *= -1
            self.y = self.y + (1 * self.y_direction)

    # Custom method for drawing the object
    def display(self):
        fill(1.0)
        ellipse(self.x_offset + self.x, self.y_offset + self.y, 6, 6)


run()
