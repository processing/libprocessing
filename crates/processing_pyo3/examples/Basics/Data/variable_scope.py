# Variable Scope.
#
# Variables have a global or local "scope".
# For example, variables declared within either the
# setup() or draw() functions may be only used in these
# functions. Global variables, variables declared outside
# of setup() and draw(), may be used anywhere within the program.
# If a local variable is declared with the same name as a
# global variable, the program will use the local variable to make
# its calculations within the current scope. In Python a variable
# is local to the whole function it is assigned in, not to a block.
from mewnala import *

a = 80  # Create a global variable "a"


def setup():
    size(640, 360)
    background(0.0)
    stroke(1.0)
    no_loop()


def draw():
    # Draw a line using the global variable "a"
    line(a, 0, a, height)

    # Make a call to the custom function draw_local_lines()
    draw_local_lines()

    # Make a call to the custom function draw_another_line()
    draw_another_line()

    # Make a call to the custom function draw_yet_another_line()
    draw_yet_another_line()


def draw_local_lines():
    # Create a new variable "a" local to this function with the for statement
    for a in range(120, 200, 2):
        line(a, 0, a, height)

    # Assign a new value to the local variable "a"
    a = 300
    # Draw a line using the local variable "a"
    line(a, 0, a, height)


def draw_another_line():
    # Create a new variable "a" local to this function
    a = 320
    # Draw a line using the local variable "a"
    line(a, 0, a, height)


def draw_yet_another_line():
    # Because no new local variable "a" is set,
    # this line draws using the original global
    # variable "a", which is set to the value 80.
    line(a + 2, 0, a + 2, height)


run()
