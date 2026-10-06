# LoadFile 1
#
# Loads a text file that contains two numbers separated by a tab ('\t').
# A new pair of numbers is loaded each frame and used to draw a point on the screen.
from mewnala import *
import os

lines = []
index = 0


def setup():
    global lines
    size(640, 360)
    background(0.0)
    stroke(1.0)
    frame_rate(12)
    # open() resolves against the cwd, so build the path from this file
    path = os.path.join(os.path.dirname(__file__), "data", "positions.txt")
    with open(path) as f:
        lines = f.read().splitlines()


def draw():
    global index
    if index < len(lines):
        pieces = lines[index].split("\t")
        if len(pieces) == 2:
            # Scale the coordinates to match the size of the sketch window
            x = remap(float(pieces[0]), 0, 100, 0, width)
            y = remap(float(pieces[1]), 0, 100, 0, height)
            point(x, y)
        # Go to the next line for the next run through draw()
        index = index + 1


run()
