# Keyboard Functions
# by Martin Gomez
#
# Click on the window to give it focus and press the letter keys to type colors.
# The keyboard function key_pressed() is called whenever
# a key is pressed. key_released() is another keyboard
# function that is called when a key is released.
#
# Original 'Color Typewriter' concept by John Maeda.
from mewnala import *

max_height = 40
min_height = 20
letter_height = max_height  # Height of the letters
letter_width = 20  # Width of the letter

x = -letter_width  # X position of the letters
y = 0  # Y position of the letters

newletter = False

num_chars = 26  # There are 26 characters in the alphabet
colors = []


def setup():
    size(640, 360)
    no_stroke()
    background(0.5)
    # Set a hue value for each key
    for i in range(num_chars):
        colors.append(hsva(i / num_chars * 360, 1.0, 1.0))


def draw():
    global newletter
    if newletter:
        # Draw the "letter"
        if letter_height == max_height:
            y_pos = y
            rect(x, y_pos, letter_width, letter_height)
        else:
            y_pos = y + min_height
            rect(x, y_pos, letter_width, letter_height)
            fill(0.5)
            rect(x, y_pos - min_height, letter_width, letter_height)
        newletter = False


def key_pressed():
    global letter_height, newletter, x, y
    # If the key is between 'A' and 'Z' or 'a' and 'z'
    if key is not None and ("A" <= key <= "Z" or "a" <= key <= "z"):
        if key <= "Z":
            key_index = ord(key) - ord("A")
            letter_height = max_height
            fill(colors[key_index])
        else:
            key_index = ord(key) - ord("a")
            letter_height = min_height
            fill(colors[key_index])
    else:
        fill(0.0)
        letter_height = 10

    newletter = True

    # Update the "letter" position
    x = x + letter_width

    # Wrap horizontally
    if x > width - letter_width:
        x = 0
        y += max_height

    # Wrap vertically
    if y > height - letter_height:
        y = 0  # reset y to 0


run()
