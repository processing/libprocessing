# Characters Strings.
#
# The character datatype, abbreviated as char, stores letters and
# symbols in the Unicode format, a coding system developed to support
# a variety of world languages. Characters are distinguished from other
# symbols by putting them between single quotes ('P').
#
# A string is a sequence of characters. A string is noted by surrounding
# a group of letters with double quotes ("Processing").
# Chars and strings are most often used with the keyboard methods,
# to display text to the screen, and to load images or files.
#
# A string is actually a class with its own methods, some of which are
# featured below.
from mewnala import *

letter = ""
words = "Begin..."


def setup():
    size(640, 360)
    # Create the font
    text_font(load_font("data/SourceCodePro-Regular.ttf"))
    text_size(36)


def draw():
    background(0.0)  # Set background to black

    # Draw the letter to the center of the screen
    text_size(14)
    text("Click on the program, then type to add to the String", 50, 50)
    text("Current key: " + letter, 50, 70)
    text("The String is " + str(len(words)) + " characters long", 50, 90)

    text_size(36)
    text(words, 50, 120, 540, 300)


def key_typed():
    global letter, words
    # The variable "key" always contains the value
    # of the most recent key pressed.
    if key is not None and ("A" <= key <= "z" or key == " "):
        letter = key
        words = words + key
        # Write the letter to the console
        print(key)


run()
