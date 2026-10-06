# Redraw.
#
# The redraw() function makes draw() execute once.
# In this example, draw() is executed once every time
# the mouse is clicked.
from mewnala import *

y = 0.0


# The statements in the setup() function
# execute once when the program begins
def setup():
    global y
    size(640, 360)  # Size should be the first statement
    stroke(1.0)  # Set line drawing color to white
    no_loop()
    y = height * 0.5


# The code in draw() is run until the program
# is stopped. Each statement is executed in
# sequence and after the last line is read,
# the first line is run again.
def draw():
    global y
    background(0.0)  # Set the background to black
    y = y - 4
    if y < 0:
        y = height
    line(0, y, width, y)


def mouse_pressed():
    redraw()


run()
