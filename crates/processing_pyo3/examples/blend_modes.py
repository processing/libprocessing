from mewnala import *

MODES = [BLEND, ADD, SUBTRACT, DARKEST, LIGHTEST, DIFFERENCE, EXCLUSION, MULTIPLY, SCREEN, REPLACE]
index = 0

def setup():
    size(500, 500)

def draw():
    global index

    if key_just_pressed(RIGHT_ARROW) or key_just_pressed(SPACE):
        index = (index + 1) % len(MODES)
    elif key_just_pressed(LEFT_ARROW):
        index = (index - 1) % len(MODES)

    background(38)
    no_stroke()
    blend_mode(MODES[index])

    fill(230, 51, 51, 191)
    rect(80, 100, 200, 250)

    fill(51, 204, 51, 191)
    rect(180, 80, 200, 250)

    fill(51, 77, 230, 191)
    rect(130, 200, 200, 200)

run()
