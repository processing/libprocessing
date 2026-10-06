# CountingString example
# by Daniel Shiffman.
#
# This example demonstrates how to use a dict to store
# a number associated with a string.
#
# This example uses the dict to perform a simple concordance
# http://en.wikipedia.org/wiki/Concordance_(publishing)
from mewnala import *
import os

# A dict pairs strings with integers
concordance = {}

# The raw list of words
tokens = []
counter = 0


def setup():
    global tokens
    size(640, 360)

    # Load file and chop it up (open() resolves against the cwd, so build the path from this file)
    path = os.path.join(os.path.dirname(__file__), "data", "dracula.txt")
    with open(path) as f:
        lines = f.read().splitlines()
    all_text = " ".join(lines).lower()
    for d in ",.?!:;[]-\"":
        all_text = all_text.replace(d, " ")
    tokens = all_text.split()

    # Create the font
    text_font(load_font("data/SourceCodePro-Regular.ttf"))
    text_size(24)


def draw():
    global counter
    background(0.2)
    fill(1.0)

    # Look at words one at a time
    if counter < len(tokens):
        s = tokens[counter]
        counter += 1
        concordance[s] = concordance.get(s, 0) + 1

    # x and y will be used to locate each word
    x = 0
    y = 48

    # The keys sorted by their count
    keys = sorted(concordance, key=concordance.get)

    # Look at each word
    for word in keys:
        count = concordance[word]

        # Only display words that appear 3 times
        if count > 3:
            # The size is the count
            fsize = constrain(count, 0, 48)
            text_size(fsize)
            text(word, x, y)
            # Move along the x-axis
            x += text_width(word + " ")

        # If x gets to the end, move y
        if x > width:
            x = 0
            y += 48
            # If y gets to the end, we're done
            if y > height:
                break


run()
