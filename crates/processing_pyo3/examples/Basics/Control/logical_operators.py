# Logical Operators.
#
# The logical operators for AND (and) and OR (or) are used to
# combine simple relational statements into more complex expressions.
# The NOT (not) operator is used to negate a boolean statement.
from mewnala import *

size(640, 360)
background(0.49)

test = False

for i in range(5, height + 1, 5):
    # Logical AND
    stroke(0.0)
    if (i > 35) and (i < 100):
        line(width / 4, i, width / 2, i)
        test = False

    # Logical OR
    stroke(0.3)
    if (i <= 35) or (i >= 100):
        line(width / 2, i, width, i)
        test = True

    # Testing if a boolean value is "true"
    # The expression "if test:" is equivalent to "if test == True:"
    if test:
        stroke(0.0)
        point(width / 3, i)

    # Testing if a boolean value is "false"
    # The expression "if not test:" is equivalent to "if test == False:"
    if not test:
        stroke(1.0)
        point(width / 4, i)

run()
