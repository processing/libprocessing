"""Drawing type through the global text API (text, text_size, text_align, ...)."""
from mewnala import *
from math import sin


def setup():
    size(640, 360)


def draw():
    background(16, 16, 24)

    # Centered title.
    fill(255)
    text_align(CENTER, CENTER)
    text_size(56)
    text("hello, processing", width / 2, height / 2)

    # Pulsing subtitle.
    pulse = 150 + 105 * sin(frame_count * 0.05)
    fill(120, pulse, 255)
    text_size(20)
    text("global text now works", width / 2, height / 2 + 60)

    # Bottom-left frame counter, left/baseline aligned.
    fill(180)
    text_align(LEFT, BASELINE)
    text_size(14)
    text(f"frame {frame_count}", 16, height - 16)


# TODO: this should happen implicitly on module load somehow
run()
