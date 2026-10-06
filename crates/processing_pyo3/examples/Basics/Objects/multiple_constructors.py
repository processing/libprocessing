# Multiple constructors
#
# A class can have multiple constructors that assign the fields in different ways.
# Sometimes it's beneficial to specify every aspect of an object's data by assigning
# parameters to the fields, but other times it might be appropriate to define only
# one or a few.
from mewnala import *

sp1 = None
sp2 = None


def setup():
    global sp1, sp2
    size(640, 360)
    background(0.8)
    no_loop()
    # Run the constructor without parameters
    sp1 = Spot()
    # Run the constructor with three parameters
    sp2 = Spot(width * 0.5, height * 0.5, 120)


def draw():
    sp1.display()
    sp2.display()


class Spot:
    # The parameters are optional; without them
    # the fields are assigned default values
    def __init__(self, xpos=None, ypos=None, r=None):
        if xpos is None:
            self.radius = 40
            self.x = width * 0.25
            self.y = height * 0.5
        else:
            self.x = xpos
            self.y = ypos
            self.radius = r

    def display(self):
        ellipse(self.x, self.y, self.radius * 2, self.radius * 2)


run()
