# No Loop.
#
# The no_loop() function causes draw() to only run once.
# Without calling no_loop(), the code inside draw() is
# run continually.
from mewnala import *

y = 180


# The statements in the setup() block
# run once when the program begins
def setup():
    size(640, 360)  # Size should be the first statement
    stroke(1.0)  # Set line drawing color to white
    no_loop()


# In this example, the code in the draw() block
# runs only once because of the no_loop() in setup()
def draw():
    global y
    background(0.0)  # Set the background to black
    line(0, y, width, y)
    y = y - 1
    if y < 0:
        y = height


run()
