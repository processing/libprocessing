# Blending
# by Andres Colubri.
#
# Images can be blended using one of the 10 blending modes.
# Click to go to cycle through the modes.
from mewnala import *

img1 = None
img2 = None
sel_mode = REPLACE
name = "REPLACE"
pic_alpha = 1.0


def setup():
    global img1, img2
    size(640, 360)
    img1 = load_image("data/layer1.jpg")
    img2 = load_image("data/layer2.jpg")
    no_stroke()


def draw():
    global pic_alpha

    pic_alpha = remap(mouse_x, 0, width, 0, 1.0)

    background(0.0)

    tint(1.0, 1.0)
    image(img1, 0, 0)

    blend_mode(sel_mode)
    tint(1.0, pic_alpha)
    image(img2, 0, 0)

    blend_mode(REPLACE)
    fill(1.0)
    rect(0, 0, 94, 22)
    fill(0.0)
    text(name, 10, 15)


def mouse_pressed():
    global sel_mode, name

    if sel_mode == REPLACE:
        sel_mode = BLEND
        name = "BLEND"
    elif sel_mode == BLEND:
        sel_mode = ADD
        name = "ADD"
    elif sel_mode == ADD:
        sel_mode = SUBTRACT
        name = "SUBTRACT"
    elif sel_mode == SUBTRACT:
        sel_mode = LIGHTEST
        name = "LIGHTEST"
    elif sel_mode == LIGHTEST:
        sel_mode = DARKEST
        name = "DARKEST"
    elif sel_mode == DARKEST:
        sel_mode = DIFFERENCE
        name = "DIFFERENCE"
    elif sel_mode == DIFFERENCE:
        sel_mode = EXCLUSION
        name = "EXCLUSION"
    elif sel_mode == EXCLUSION:
        sel_mode = MULTIPLY
        name = "MULTIPLY"
    elif sel_mode == MULTIPLY:
        sel_mode = SCREEN
        name = "SCREEN"
    elif sel_mode == SCREEN:
        sel_mode = REPLACE
        name = "REPLACE"


def mouse_dragged():
    global pic_alpha
    if height - 50 < mouse_y:
        pic_alpha = remap(mouse_x, 0, width, 0, 1.0)


run()
