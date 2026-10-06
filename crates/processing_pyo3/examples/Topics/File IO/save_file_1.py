# SaveFile 1
#
# Saving files is a useful way to store data so it can be viewed after a
# program has stopped running. Writing a list of strings to a file, with
# each string written to a new line, saves it to the sketch's folder.
from mewnala import *
import os

x = []
y = []


def setup():
    size(640, 360)


def draw():
    background(0.8)
    stroke(0.0)
    no_fill()
    begin_shape()
    for i in range(len(x)):
        vertex(x[i], y[i])
    end_shape()
    # Show the next segment to be added
    if len(x) >= 1:
        stroke(1.0)
        line(mouse_x, mouse_y, x[len(x) - 1], y[len(y) - 1])


def mouse_pressed():  # Click to add a line segment
    x.append(int(mouse_x))
    y.append(int(mouse_y))


def key_pressed():  # Press a key to save the data
    lines = []
    for i in range(len(x)):
        lines.append(str(x[i]) + "\t" + str(y[i]))
    # Write next to this sketch (open() resolves against the cwd)
    path = os.path.join(os.path.dirname(__file__), "lines.txt")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")
    exit_sketch()  # Stop the program


run()
