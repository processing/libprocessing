# Wave Gradient
# by Ira Greenberg.
#
# Generate a gradient along a sin() wave.
from mewnala import *

amplitude = 30
fill_gap = 2.5
gradient = None


def setup():
    global gradient
    size(640, 360)
    # To efficiently set all the pixels on screen, make the set()
    # calls on an Image, then write the result to the screen.
    gradient = create_image(width, height)
    no_loop()


def draw():
    background(0.78)
    frequency = 0

    for i in range(-75, height + 75):
        # Reset angle to 0, so waves stack properly
        angle = 0
        # Increasing frequency causes more gaps
        frequency += 0.002
        for j in range(width + 75):
            py = i + sin(radians(angle)) * amplitude
            angle += frequency
            c = color(abs(py - i) / amplitude, 1 - abs(py - i) / amplitude, j / (width + 50))
            # Hack to fill gaps. Raise value of fill_gap if you increase frequency
            filler = 0
            while filler < fill_gap:
                gradient.set(int(j - filler), int(py) - filler, c)  # GAP: Image.set raises on out-of-range coordinates (Processing ignores them) and fails in the frame the image was created
                gradient.set(int(j), int(py), c)
                gradient.set(int(j + filler), int(py) + filler, c)
                filler += 1
    # Draw the image to the screen
    image(gradient, 0, 0)


run()
