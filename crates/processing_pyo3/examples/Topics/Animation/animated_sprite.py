# Animated Sprite (Shifty + Teddy)
# by James Paterson.
#
# Press the mouse button to change animations.
# Demonstrates loading, displaying, and animating GIF images.
# It would be easy to write a program to display
# animated GIFs, but would not allow as much control over
# the display sequence and rate of display.
from mewnala import *

animation1 = None
animation2 = None

xpos = 0.0
ypos = 0.0
drag = 30.0


def setup():
    global animation1, animation2, ypos
    size(640, 360)
    background(1.0, 0.8, 0.0)
    frame_rate(24)
    animation1 = Animation("data/PT_Shifty_", 38)
    animation2 = Animation("data/PT_Teddy_", 60)
    ypos = height * 0.25


def draw():
    global xpos
    dx = mouse_x - xpos
    xpos = xpos + dx / drag

    # Display the sprite at the position xpos, ypos
    if mouse_is_pressed:
        background(0.6, 0.6, 0.0)
        animation1.display(xpos - animation1.get_width() / 2, ypos)
    else:
        background(1.0, 0.8, 0.0)
        animation2.display(xpos - animation1.get_width() / 2, ypos)


# Class for animating a sequence of images
class Animation:
    def __init__(self, image_prefix, count):
        self.image_count = count
        self.images = []
        self.frame = 0

        for i in range(self.image_count):
            # zfill() formats i into four digits
            filename = image_prefix + str(i).zfill(4) + ".png"
            self.images.append(load_image(filename))

    def display(self, xpos, ypos):
        self.frame = (self.frame + 1) % self.image_count
        image(self.images[self.frame], xpos, ypos)

    def get_width(self):
        return self.images[0].width


run()
