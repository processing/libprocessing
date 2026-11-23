# P_2_3_3_01_TABLET_TOOL
# drawItem
#
# Generative Gestaltung, ISBN: 978-3-87439-759-9
# First Edition, Hermann Schmidt, Mainz, 2009
# Hartmut Bohnacker, Benedikt Gross, Julia Laub, Claudius Lazzeroni
# Copyright 2009 Hartmut Bohnacker, Benedikt Gross, Julia Laub, Claudius Lazzeroni
#
# http://www.generative-gestaltung.de
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

#
# Ported from the Processing original to processing_py.

# this sketch does more less the same like P_2_3_3_01_TABLET.pde ...
# but it shows how to implement an undo feature. not really for beginners.
#
# The generativedesign Tablet of the original is replaced by a small class
# that reads the mouse and reports a constant pressure.
#
# MOUSE
# drag                : draw with text
#
# TABLET
# pressure            : textsize
#
# KEYS
# del, backspace      : clear screen
# shift               : limit drawing direction
# z                   : undo last action
# u                   : show underlay picture
# arrow up            : angle distortion +
# arrow down          : angle distortion -
# s                   : save png
from datetime import datetime

from processing_py import *


class Tablet:
    # the mouse has no pressure, so a constant value is reported
    def get_pressure(self):
        return 0.5


class DrawItem:
    def __init__(self):
        self.x = 0
        self.y = 0
        self.angle = 0
        self.font_size = 0
        self.letter = " "

    def draw(self):
        push_matrix()
        translate(self.x, self.y)
        rotate(self.angle)
        text_font(font)
        text_size(self.font_size)
        text_align(LEFT)
        fill(0.0)
        text(self.letter, 0, 0)
        pop_matrix()


tablet = None

x = 0
y = 0
step_size = 5.0

font = None
letters = "Sie hören nicht die folgenden Gesänge, Die Seelen, denen ich die ersten sang, Zerstoben ist das freundliche Gedränge, Verklungen ach! der erste Wiederklang."
font_size_min = 5
angle_distortion = 0.0

counter = 0

draw_items = []
undo_index = 0

show_undelay = False
img = None
image_alpha = 100 / 255

# click_pos_x = 0
# click_pos_y = 0


def setup():
    global tablet, draw_items, font, img
    # use fullscreen size
    size(display_width, display_height)  # GAP: display_width / display_height are 0 until the first draw(), so the window cannot be created
    background(1.0)

    tablet = Tablet()
    draw_items = []

    # font = create_font("Arial")
    font = create_font("ArnhemFineTT-Normal")  # GAP: create_font() raises "Font not found" for a font that is not installed (Processing falls back to the default font)
    text_font(font)
    text_size(font_size_min)
    cursor(CROSS)

    # load an image in background
    img = load_image("data/underlay.png")


def draw():
    global x, y, step_size, counter
    if mouse_is_pressed:
        # gamma values optimized for wacom intuos 3
        pressure = gamma(tablet.get_pressure() * 1.1, 2.5)

        d = dist(x, y, mouse_x, mouse_y)
        new_font_size = font_size_min + 200 * pressure
        text_font(font)
        text_size(new_font_size)
        new_letter = letters[counter]
        step_size = text_width(new_letter)

        angle = atan2(mouse_y - y, mouse_x - x)

        if key_is_pressed and key_code == SHIFT:
            angle = round(8 * angle / TWO_PI)
            if angle % 2 == 0:
                angle = angle * TWO_PI / 8
            else:
                d = 0

        if d > step_size:
            angle += random(angle_distortion)

            drw = DrawItem()
            drw.angle = angle
            drw.x = x
            drw.y = y
            drw.letter = new_letter
            drw.font_size = new_font_size
            drw.draw()
            draw_items.append(drw)

            counter += 1
            if counter > len(letters) - 1:
                counter = 0

            x = x + cos(angle) * step_size
            y = y + sin(angle) * step_size


def mouse_pressed():
    global undo_index, x, y
    undo_index = len(draw_items)
    x = mouse_x
    y = mouse_y

    # remember click position
    # click_pos_x = mouse_x
    # click_pos_y = mouse_y


def key_released():
    global draw_items, undo_index, show_undelay
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)
        draw_items = []
        undo_index = 0
    if key == "u" or key == "U":
        show_undelay = not show_undelay
    if show_undelay:
        background(1.0)
        re_draw_all_draw_items()
        tint(1.0, image_alpha)
        image(img, 0, 0)
    else:
        re_draw_all_draw_items()

    if key == "z" or key == "Z":
        undo_index = undo_draw_items(undo_index)
        re_draw_all_draw_items()


def key_pressed():
    global angle_distortion
    # angle_distortion ctrls arrowkeys up/down
    if key_code == UP:
        angle_distortion += 0.1
    if key_code == DOWN:
        angle_distortion -= 0.1


# gamma ramp, non linaer mapping ...
def gamma(the_value, the_gamma):
    return pow(the_value, the_gamma)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


def re_draw_all_draw_items():
    background(1.0)
    for i in range(len(draw_items)):
        tmp = draw_items[i]
        tmp.draw()


def undo_draw_items(the_undo_index):
    the_undo_index -= 1
    if len(draw_items) > 0 and the_undo_index >= 0:
        for i in range(len(draw_items) - 1, the_undo_index, -1):
            draw_items.pop(i)
    return the_undo_index


run()
