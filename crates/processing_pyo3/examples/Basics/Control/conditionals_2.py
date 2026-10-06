# Conditionals 2.
#
# We extend the language of conditionals from the previous
# example by adding the keyword "else". This allows conditionals
# to ask two or more sequential questions, each with a different
# action.
from mewnala import *

size(640, 360)
background(0.0)

for i in range(2, width - 2, 2):
    # If 'i' divides by 20 with no remainder
    if (i % 20) == 0:
        stroke(1.0)
        line(i, 80, i, height / 2)
    # If 'i' divides by 10 with no remainder
    elif (i % 10) == 0:
        stroke(0.6)
        line(i, 20, i, 180)
    # If neither of the above two conditions are met
    # then draw this line
    else:
        stroke(0.4)
        line(i, height / 2, i, height - 20)

run()
