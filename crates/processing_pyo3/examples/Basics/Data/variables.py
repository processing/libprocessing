# Variables.
#
# Variables are used for storing values. In this example, change
# the values of variables to affect the composition.
from mewnala import *

size(640, 360)
background(0.0)
stroke(0.6)
stroke_weight(4)
stroke_cap(SQUARE)

a = 50
b = 120
c = 180

line(a, b, a + c, b)
line(a, b + 10, a + c, b + 10)
line(a, b + 20, a + c, b + 20)
line(a, b + 30, a + c, b + 30)

a = a + c
b = height - b

line(a, b, a + c, b)
line(a, b + 10, a + c, b + 10)
line(a, b + 20, a + c, b + 20)
line(a, b + 30, a + c, b + 30)

a = a + c
b = height - b

line(a, b, a + c, b)
line(a, b + 10, a + c, b + 10)
line(a, b + 20, a + c, b + 20)
line(a, b + 30, a + c, b + 30)

run()
