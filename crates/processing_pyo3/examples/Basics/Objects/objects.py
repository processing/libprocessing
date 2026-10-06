# Objects
# by hbarragan.
#
# Move the cursor across the image to change the speed and positions
# of the geometry. The class MRect defines a group of lines.
from mewnala import *

r1 = None
r2 = None
r3 = None
r4 = None


def setup():
    global r1, r2, r3, r4
    size(640, 360)
    fill(1.0, 0.8)
    no_stroke()
    r1 = MRect(1, 134.0, 0.532, 0.1 * height, 10.0, 60.0)
    r2 = MRect(2, 44.0, 0.166, 0.3 * height, 5.0, 50.0)
    r3 = MRect(2, 58.0, 0.332, 0.4 * height, 10.0, 35.0)
    r4 = MRect(1, 120.0, 0.0498, 0.9 * height, 15.0, 60.0)


def draw():
    background(0.0)

    r1.display()
    r2.display()
    r3.display()
    r4.display()

    r1.move(mouse_x - (width / 2), mouse_y + (height * 0.1), 30)
    r2.move((mouse_x + (width * 0.05)) % width, mouse_y + (height * 0.025), 20)
    r3.move(mouse_x / 4, mouse_y - (height * 0.025), 40)
    r4.move(mouse_x - (width / 2), (height - mouse_y), 50)


class MRect:
    def __init__(self, iw, ixp, ih, iyp, id, it):
        self.w = iw  # single bar width
        self.xpos = ixp  # rect xposition
        self.h = ih  # rect height
        self.ypos = iyp  # rect yposition
        self.d = id  # single bar distance
        self.t = it  # number of bars

    def move(self, pos_x, pos_y, damping):
        dif = self.ypos - pos_y
        if abs(dif) > 1:
            self.ypos -= dif / damping
        dif = self.xpos - pos_x
        if abs(dif) > 1:
            self.xpos -= dif / damping

    def display(self):
        for i in range(int(self.t)):
            rect(self.xpos + (i * (self.d + self.w)), self.ypos, self.w, height * self.h)


run()
