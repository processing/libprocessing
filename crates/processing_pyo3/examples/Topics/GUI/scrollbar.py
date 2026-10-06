# Scrollbar.
#
# Move the scrollbars left and right to change the positions of the images.
from mewnala import *

# True if a mouse button was pressed while no other button was.
first_mouse_press = False
hs1 = None  # Two scrollbars
hs2 = None
img1 = None  # Two images to load
img2 = None


def setup():
    global hs1, hs2, img1, img2
    size(640, 360)
    no_stroke()

    hs1 = HScrollbar(0, height / 2 - 8, width, 16, 16)
    hs2 = HScrollbar(0, height / 2 + 8, width, 16, 16)

    # Load images
    img1 = load_image("data/seedTop.jpg")
    img2 = load_image("data/seedBottom.jpg")


def draw():
    global first_mouse_press
    background(1.0)

    # Get the position of the img1 scrollbar
    # and convert to a value to display the img1 image
    img1_pos = hs1.get_pos() - width / 2
    fill(1.0)
    image(img1, width / 2 - img1.width / 2 + img1_pos * 1.5, 0)

    # Get the position of the img2 scrollbar
    # and convert to a value to display the img2 image
    img2_pos = hs2.get_pos() - width / 2
    fill(1.0)
    image(img2, width / 2 - img2.width / 2 + img2_pos * 1.5, height / 2)

    hs1.update()
    hs2.update()
    hs1.display()
    hs2.display()

    stroke(0.0)
    line(0, height / 2, width, height / 2)

    # After it has been used in the sketch, set it back to false
    if first_mouse_press:
        first_mouse_press = False


def mouse_pressed():
    global first_mouse_press
    if not first_mouse_press:
        first_mouse_press = True


class HScrollbar:
    def __init__(self, xp, yp, sw, sh, l):
        self.swidth = sw  # width and height of bar
        self.sheight = sh
        widthtoheight = sw - sh
        self.ratio = sw / widthtoheight
        self.xpos = xp  # x and y position of bar
        self.ypos = yp - self.sheight / 2
        self.spos = self.xpos + self.swidth / 2 - self.sheight / 2  # x position of slider
        self.newspos = self.spos
        self.spos_min = self.xpos  # max and min values of slider
        self.spos_max = self.xpos + self.swidth - self.sheight
        self.loose = l  # how loose/heavy
        self.over = False  # is the mouse over the slider?
        self.locked = False

    def update(self):
        if self.over_event():
            self.over = True
        else:
            self.over = False
        if first_mouse_press and self.over:
            self.locked = True
        if not mouse_is_pressed:
            self.locked = False
        if self.locked:
            self.newspos = self.constrain(mouse_x - self.sheight / 2, self.spos_min, self.spos_max)
        if abs(self.newspos - self.spos) > 1:
            self.spos = self.spos + (self.newspos - self.spos) / self.loose

    def constrain(self, val, minv, maxv):
        return min(max(val, minv), maxv)

    def over_event(self):
        if self.xpos < mouse_x < self.xpos + self.swidth and self.ypos < mouse_y < self.ypos + self.sheight:
            return True
        else:
            return False

    def display(self):
        no_stroke()
        fill(0.8)
        rect(self.xpos, self.ypos, self.swidth, self.sheight)
        if self.over or self.locked:
            fill(0.0, 0.0, 0.0)
        else:
            fill(0.4, 0.4, 0.4)
        rect(self.spos, self.ypos, self.sheight, self.sheight)

    def get_pos(self):
        # Convert spos to be values between
        # 0 and the total width of the scrollbar
        return self.spos * self.ratio


run()
