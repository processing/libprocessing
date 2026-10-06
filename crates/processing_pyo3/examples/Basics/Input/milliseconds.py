# Milliseconds.
#
# A millisecond is 1/1000 of a second.
# Processing keeps track of the number of milliseconds a program has run.
# By modifying this number with the modulo(%) operator,
# different patterns in time are created.
from mewnala import *

unit = 0


def setup():
    global unit
    size(640, 360)
    no_stroke()
    unit = width // 20


def draw():
    for i in range(unit):
        period = (i + 1) * unit * 10
        fill((millis() % period) / period)
        rect(i * unit, 0, unit, height)


run()
