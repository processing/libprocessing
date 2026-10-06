# Spore 1
# by Mike Davis.
#
# A short program for alife experiments. Click in the window to restart.
# Each cell is represented by a pixel on the display as well as an entry in
# the array 'cells'. Each cell has a run() method, which performs actions
# based on the cell's surroundings.  Cells run one at a time (to avoid conflicts
# like wanting to move to the same space) and in random order.
from mewnala import *

w = None
numcells = 0
maxcells = 6700
cells = []
spore_color = None
# set lower for smoother animation, higher for faster simulation
runs_per_loop = 10000
black = color(0.0, 0.0, 0.0)


def setup():
    size(640, 360)
    pixel_density(1)
    frame_rate(24)
    reset()


def reset():
    global w, spore_color
    clear_screen()
    w = World()
    spore_color = color(0.67, 1.0, 0.5)
    seed()


def seed():
    global numcells
    # Add cells at random places
    for i in range(maxcells):
        c_x = int(random(width))
        c_y = int(random(height))
        if w.getpix(c_x, c_y) == black:
            w.setpix(c_x, c_y, spore_color)
            cells.append(Cell(c_x, c_y))
            numcells += 1


def draw():
    # Run cells in random order
    for i in range(runs_per_loop):
        selected = min(int(random(numcells)), numcells - 1)
        cells[selected].run()


def clear_screen():
    background(0.0)


class Cell:
    def __init__(self, xin, yin):
        self.x = xin
        self.y = yin

    # Perform action based on surroundings
    def run(self):
        # Fix cell coordinates
        while self.x < 0:
            self.x += width
        while self.x > width - 1:
            self.x -= width
        while self.y < 0:
            self.y += height
        while self.y > height - 1:
            self.y -= height

        # Cell instructions
        if w.getpix(self.x + 1, self.y) == black:
            self.move(0, 1)
        elif w.getpix(self.x, self.y - 1) != black and w.getpix(self.x, self.y + 1) != black:
            self.move(int(random(9)) - 4, int(random(9)) - 4)

    # Will move the cell (dx, dy) units if that space is empty
    def move(self, dx, dy):
        if w.getpix(self.x + dx, self.y + dy) == black:
            w.setpix(self.x + dx, self.y + dy, w.getpix(self.x, self.y))
            w.setpix(self.x, self.y, color(0.0))
            self.x += dx
            self.y += dy


# The World class simply provides two functions, get and set, which access the
# display in the same way as getPixel and setPixel.  The only difference is that
# the World class's get and set do screen wraparound ("toroidal coordinates").
class World:
    def setpix(self, x, y, c):
        while x < 0:
            x += width
        while x > width - 1:
            x -= width
        while y < 0:
            y += height
        while y > height - 1:
            y -= height
        set(x, y, c)

    def getpix(self, x, y):
        while x < 0:
            x += width
        while x > width - 1:
            x -= width
        while y < 0:
            y += height
        while y > height - 1:
            y -= height
        return get(x, y)


def mouse_pressed():
    global numcells
    numcells = 0
    cells.clear()
    reset()


run()
