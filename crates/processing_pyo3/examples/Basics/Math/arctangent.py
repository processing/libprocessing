# Arctangent.
#
# Move the mouse to change the direction of the eyes.
# The atan2() function computes the angle from each eye
# to the cursor.
from mewnala import *

e1 = None
e2 = None
e3 = None


def setup():
    global e1, e2, e3
    size(640, 360)
    no_stroke()
    e1 = Eye(250, 16, 120)
    e2 = Eye(164, 185, 80)
    e3 = Eye(420, 230, 220)


def draw():
    background(0.4)

    e1.update(mouse_x, mouse_y)
    e2.update(mouse_x, mouse_y)
    e3.update(mouse_x, mouse_y)

    e1.display()
    e2.display()
    e3.display()


class Eye:
    def __init__(self, tx, ty, ts):
        self.x = tx
        self.y = ty
        self.size = ts
        self.angle = 0.0

    def update(self, mx, my):
        self.angle = atan2(my - self.y, mx - self.x)

    def display(self):
        push_matrix()
        translate(self.x, self.y)
        fill(1.0)
        ellipse(0, 0, self.size, self.size)
        rotate(self.angle)
        fill(0.6, 0.8, 0.0)
        ellipse(self.size / 4, 0, self.size / 2, self.size / 2)
        pop_matrix()


run()
