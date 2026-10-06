# Handles.
#
# Click and drag the white boxes to change their position.
from mewnala import *

handles = []

# True if a mouse button has just been pressed while no other button was.
first_mouse_press = False


def setup():
    size(640, 360)
    num = height // 15
    hsize = 10
    for i in range(num):
        handles.append(Handle(width // 2, 10 + i * 15, 50 - hsize // 2, 10, handles))


def draw():
    global first_mouse_press
    background(0.6)

    for i in range(len(handles)):
        handles[i].update()
        handles[i].display()

    fill(0.0)
    rect(0, 0, width / 2, height)

    # After it has been used in the sketch, set it back to false
    if first_mouse_press:
        first_mouse_press = False


def mouse_pressed():
    global first_mouse_press
    if not first_mouse_press:
        first_mouse_press = True


def mouse_released():
    for i in range(len(handles)):
        handles[i].release_event()


class Handle:
    def __init__(self, ix, iy, il, i_size, o):
        self.x = ix
        self.y = iy
        self.stretch = il
        self.size = i_size
        self.boxx = self.x + self.stretch - self.size // 2
        self.boxy = self.y - self.size // 2
        self.others = o
        self.over = False
        self.press = False
        self.locked = False
        self.otherslocked = False

    def update(self):
        self.boxx = self.x + self.stretch
        self.boxy = self.y - self.size // 2

        for i in range(len(self.others)):
            if self.others[i].locked:
                self.otherslocked = True
                break
            else:
                self.otherslocked = False

        if not self.otherslocked:
            self.over_event()
            self.press_event()

        if self.press:
            self.stretch = lock(int(mouse_x) - width // 2 - self.size // 2, 0, width // 2 - self.size - 1)

    def over_event(self):
        if over_rect(self.boxx, self.boxy, self.size, self.size):
            self.over = True
        else:
            self.over = False

    def press_event(self):
        if (self.over and first_mouse_press) or self.locked:
            self.press = True
            self.locked = True
        else:
            self.press = False

    def release_event(self):
        self.locked = False

    def display(self):
        line(self.x, self.y, self.x + self.stretch, self.y)
        fill(1.0)
        stroke(0.0)
        rect(self.boxx, self.boxy, self.size, self.size)
        if self.over or self.press:
            line(self.boxx, self.boxy, self.boxx + self.size, self.boxy + self.size)
            line(self.boxx, self.boxy + self.size, self.boxx + self.size, self.boxy)


def over_rect(x, y, w, h):
    if x <= mouse_x <= x + w and y <= mouse_y <= y + h:
        return True
    else:
        return False


def lock(val, minv, maxv):
    return min(max(val, minv), maxv)


run()
