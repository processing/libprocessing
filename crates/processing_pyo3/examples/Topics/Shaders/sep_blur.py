# Separate Blur Shader
#
# This blur shader works by applying two successive passes, one horizontal
# and the other vertical.
#
# Press the mouse to switch between the custom and default shader.
from mewnala import *

blur = None
src = None
pass1 = None
pass2 = None
blur_size = 9
sigma = 5.0


def setup():
    global blur, src, pass1, pass2
    size(640, 360)

    blur = load_shader("data/sep_blur.wesl")

    src = create_graphics(width, height)
    pass1 = create_graphics(width, height)
    pass2 = create_graphics(width, height)


def draw():
    src.begin_draw()
    src.background(0.0)
    src.fill(1.0)
    src.ellipse(width / 2, height / 2, 100, 100)
    src.end_draw()

    # Applying the blur shader along the vertical direction
    pass1.begin_draw()
    pass1.image(src, 0, 0)
    pass1.filter(blur, blur_size=blur_size, sigma=sigma, horizontal_pass=0)
    pass1.end_draw()

    # Applying the blur shader along the horizontal direction
    pass2.begin_draw()
    pass2.image(pass1, 0, 0)
    pass2.filter(blur, blur_size=blur_size, sigma=sigma, horizontal_pass=1)
    pass2.end_draw()

    image(pass2, 0, 0)


def key_pressed():
    global blur_size, sigma
    if key == "9":
        blur_size = 9
        sigma = 5.0
    elif key == "7":
        blur_size = 7
        sigma = 3.0
    elif key == "5":
        blur_size = 5
        sigma = 2.0
    elif key == "3":
        blur_size = 5
        sigma = 1.0


run()
