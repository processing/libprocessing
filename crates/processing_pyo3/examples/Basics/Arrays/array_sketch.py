# Array.
#
# An array is a list of data. Each piece of data in an array
# is identified by an index number representing its position in
# the array. Arrays are zero based, which means that the first
# element in the array is [0], the second element is [1], and so on.
# In this example, an array named "coswave" is created and
# filled with the cosine values. This data is displayed three
# separate ways on the screen.
from mewnala import *

coswave = []


def setup():
    size(640, 360)
    for i in range(width):
        amount = remap(i, 0, width, 0, PI)
        coswave.append(abs(cos(amount)))
    background(1.0)
    no_loop()


def draw():
    y1 = 0
    y2 = height // 3
    for i in range(width):
        stroke(coswave[i])
        line(i, y1, i, y2)

    y1 = y2
    y2 = y1 + y1
    for i in range(width):
        stroke(coswave[i] / 4)
        line(i, y1, i, y2)

    y1 = y2
    y2 = height
    for i in range(width):
        stroke(1.0 - coswave[i])
        line(i, y1, i, y2)


run()
