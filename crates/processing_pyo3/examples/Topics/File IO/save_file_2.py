# SaveFile 2
#
# This file uses a file object to write data continuously to a file
# while the mouse is pressed. When a key is pressed, the file closes
# itself and the program is stopped.
from mewnala import *
import os

output = None


def setup():
    global output
    size(640, 360)
    # Create a new file in the sketch directory (open() resolves against the cwd)
    output = open(os.path.join(os.path.dirname(__file__), "positions.txt"), "w")
    frame_rate(12)


def draw():
    if mouse_is_pressed:
        point(mouse_x, mouse_y)
        # Write the coordinate to a file with a
        # "\t" (TAB character) between each entry
        output.write(str(int(mouse_x)) + "\t" + str(int(mouse_y)) + "\n")


def key_pressed():  # Press a key to save the data
    output.flush()  # Write the remaining data
    output.close()  # Finish the file
    exit_sketch()  # Stop the program


run()
