# True/False.
#
# A Boolean variable has only two possible values: true or false.
# It is common to use Booleans with control statements to
# determine the flow of a program. In this example, when the
# boolean value "x" is true, vertical black lines are drawn and when
# the boolean value "x" is false, horizontal gray lines are drawn.
from mewnala import *

b = False

size(640, 360)
background(0.0)
stroke(1.0)

d = 20
middle = width // 2

for i in range(d, width + 1, d):

    if i < middle:
        b = True
    else:
        b = False

    if b == True:
        # Vertical line
        line(i, d, i, height - d)

    if b == False:
        # Horizontal line
        line(middle, i - middle + d, width - d, i - middle + d)

run()
