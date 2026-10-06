# Loop.
#
# If no_loop() is run in setup(), the code in draw()
# is only run once. In this example, click the mouse
# to run the loop() function to cause the draw() the
# run continuously.
from mewnala import *

y = 180


# The statements in the setup() function
# run once when the program begins
def setup():
    size(640, 360)  # Size should be the first statement
    stroke(1.0)  # Set stroke color to white
    no_loop()


def draw():
    global y
    background(0.0)  # Set the background to black
    line(0, y, width, y)
    y = y - 1
    if y < 0:
        y = height


def mouse_pressed():
    loop()


run()
