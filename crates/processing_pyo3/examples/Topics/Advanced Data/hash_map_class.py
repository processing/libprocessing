# HashMap example
# by Daniel Shiffman.
#
# This example demonstrates how to use a dict to store
# a collection of objects referenced by a key. This is much like a list,
# only instead of accessing elements with a numeric index, we use a string.
# If you are familiar with associative arrays from other languages,
# this is the same idea.
#
# A simpler example is CountingStrings which pairs strings with numbers.
# Here we pair a string with a custom object, in this case a "Word"
# object that stores two numbers.
#
# In this example, words that appear in one book (Dracula) only are colored white
# while words the other (Frankenstein) are colored black.
from mewnala import *
import os

words = {}  # dict object


def setup():
    size(640, 360)

    # Load two files
    load_file("dracula.txt")
    load_file("frankenstein.txt")

    # Create the font
    text_font(load_font("data/SourceCodePro-Regular.ttf"))
    text_size(24)


def draw():
    background(0.49)

    # Show words
    for w in words.values():
        if w.qualify():
            w.display()
            w.move()


# Load a file
def load_file(filename):
    # open() resolves against the cwd, so build the path from this file
    path = os.path.join(os.path.dirname(__file__), "data", filename)
    with open(path) as f:
        lines = f.read().splitlines()
    all_text = " ".join(lines).lower()
    for d in ",.?!:;[]-\"'":
        all_text = all_text.replace(d, " ")
    tokens = all_text.split()

    for s in tokens:
        # Is the word in the dict
        if s in words:
            # Get the word object and increase the count
            # We access objects from a dict via its key, the string
            w = words[s]
            # Which book am I loading?
            if "dracula" in filename:
                w.increment_dracula()
            elif "frankenstein" in filename:
                w.increment_franken()
        else:
            # Otherwise make a new word
            w = Word(s)
            # And add to the dict: the key is the string and the value is the Word object
            words[s] = w
            if "dracula" in filename:
                w.increment_dracula()
            elif "frankenstein" in filename:
                w.increment_franken()


class Word:
    def __init__(self, s):
        # Store a count for occurences in two different books
        self.count_dracula = 0
        self.count_franken = 0
        # Also the total count
        self.total_count = 0
        # What is the string
        self.word = s
        # Where is it on the screen
        self.position = Vec2(random(width), random(-height, height * 2))

    # We will display a word if it appears at least 5 times
    # and only in one of the books
    def qualify(self):
        if (self.count_dracula == self.total_count or self.count_franken == self.total_count) and self.total_count > 5:
            return True
        else:
            return False

    # Increment the count for Dracula
    def increment_dracula(self):
        self.count_dracula += 1
        self.total_count += 1

    # Increment the count for Frankenstein
    def increment_franken(self):
        self.count_franken += 1
        self.total_count += 1

    # The more often it appears, the faster it falls
    def move(self):
        speed = remap(self.total_count, 5, 25, 0.1, 0.4)
        speed = constrain(speed, 0, 10)
        self.position.y += speed

        if self.position.y > height * 2:
            self.position.y = -height

    # Depending on which book it gets a color
    def display(self):
        if self.count_dracula > 0:
            fill(1.0)
        elif self.count_franken > 0:
            fill(0.0)
        # Its size is also tied to number of occurences
        fs = remap(self.total_count, 5, 25, 2, 24)
        fs = constrain(fs, 2, 48)
        text_size(fs)
        text_align(CENTER)
        text(self.word, self.position.x, self.position.y)


run()
