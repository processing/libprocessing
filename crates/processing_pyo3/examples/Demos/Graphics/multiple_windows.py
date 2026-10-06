# MultipleWindows, by Andres Colubri
# Adapted from a PixelFlow example by Thomas Diewald
#
# Demonstration of a multiple window sketch, including resource
# sharing across windows. Press a key to open the other windows.
from mewnala import *

child_a = None
child_b = None
child_c = None


def setup():
    global child_a
    size(400, 300)
    print("Creating window 1")

    child_a = ChildApplet(2, 500, 0, 400, 300)
    child_a.bck_color = color(0.89, 0.68, 0.15)

    # Change location of parent window after creating child window.
    window_move(100, 0)


def draw():
    background(0.13)

    fill(0.63)
    text_align(CENTER, CENTER)
    text("MAIN window", width / 2, height / 2)

    translate(mouse_x, mouse_y)
    pointer()

    txt = "Window 1   %6.2fps" % frame_rate()
    window_title(txt)

    for child in (child_a, child_b, child_c):
        if child is not None:
            child.draw()


def key_pressed():
    global child_b, child_c
    if child_b is None:
        child_b = ChildApplet(3, 500, 353, 400, 300)
        child_b.bck_color = color(0.54, 0.89, 0.15)
    elif child_c is None:
        child_c = ChildApplet(4, 100, 353, 400, 300)
        child_c.bck_color = color(0.2, 0.62, 0.82)


# The shared pointer shape, drawn in any window
def pointer(g=None):
    if g is None:
        ellipse(0, 0, 20, 20)
    else:
        g.ellipse(0, 0, 20, 20)


class ChildApplet:
    def __init__(self, id, vx, vy, vw, vh):
        self.id = id
        self.vx = vx
        self.vy = vy
        self.vw = vw
        self.vh = vh
        self.bck_color = color(0.0)

        print("Creating window " + str(id))
        self.g = create_window(vw, vh, "Window " + str(id))
        self.g.window_move(vx, vy)  # GAP: no window_move() for secondary windows
        self.g.window_resizable(True)  # GAP: no window_resizable() for secondary windows

    def draw(self):
        g = self.g
        g.background(self.bck_color)
        g.fill(1.0)
        g.text_align(CENTER, CENTER)
        g.text("CHILD window " + str(self.id), g.width / 2, g.height / 2)

        g.push_matrix()
        g.translate(g.mouse_x, g.mouse_y)
        pointer(g)
        g.pop_matrix()

        txt = "Window %d   %6.2fps" % (self.id, frame_rate())
        g.window_title(txt)  # GAP: no window_title() for secondary windows


run()
