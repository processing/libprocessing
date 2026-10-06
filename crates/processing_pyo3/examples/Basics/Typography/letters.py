# Letters.
#
# Draws letters to the screen. This requires loading a font,
# setting the font, and then drawing the letters.
from mewnala import *

f = None


def setup():
    global f
    size(640, 360)
    background(0.0)

    # Create the font
    print(list_fonts())
    f = load_font("data/SourceCodePro-Regular.ttf")
    text_font(f)
    text_size(24)
    text_align(CENTER, CENTER)


def draw():
    background(0.0)

    # Set the left and top margin
    margin = 10
    translate(margin * 4, margin * 4)

    gap = 46
    counter = 35

    for y in range(0, height - gap, gap):
        for x in range(0, width - gap, gap):

            letter = chr(counter)

            if letter == "A" or letter == "E" or letter == "I" or letter == "O" or letter == "U":
                fill(1.0, 0.8, 0.0)
            else:
                fill(1.0)

            # Draw the letter to the screen
            text(letter, x, y)

            # Increment the counter
            counter += 1


run()
