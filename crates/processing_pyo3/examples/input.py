from mewnala import *

def setup():
    size(400, 400)

def draw():
    background(220)
    no_stroke()

    if mouse_is_pressed:
        fill(200, 50, 50)
    else:
        fill(50, 130, 200)

    rect(mouse_x - 25, mouse_y - 25, 50, 50)

    if key_is_pressed:
        if key_is_down(ESCAPE):
            return

run()
