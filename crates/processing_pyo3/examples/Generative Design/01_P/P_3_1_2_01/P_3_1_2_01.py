# P_3_1_2_01
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

# typewriter. uses input (text) as blueprint for a visual composition.
#
# MOUSE
# drag                : move canvas
#
# KEYS
# a-z                 : text input (keyboard)
# ,.!? and return     : curves
# space               : small curve with random direction
# del, backspace      : remove last letter
# arrow up            : zoom canvas +
# arrow down          : zoom canvas -
# alt                 : new random layout
# ctrl                : save png
from datetime import datetime

from processing_py import *

save_png = False

font = None
text_typed = ""

shape_space = None
shape_space2 = None
shape_period = None
shape_comma = None
shape_questionmark = None
shape_exclamationmark = None
shape_return = None

center_x = 0
center_y = 0
offset_x = 0
offset_y = 0
zoom = 0.75

act_random_seed = 6


def setup():
    global text_typed, center_x, center_y, font
    global shape_space, shape_space2, shape_period, shape_comma
    global shape_questionmark, shape_exclamationmark, shape_return
    size(display_width, display_height)  # GAP: display_width / display_height are 0 inside setup(), so the window cannot be created
    # make window resizable
    window_resizable(True)

    # text to begin with
    text_typed += "Ich bin der Musikant mit Taschenrechner in der Hand!\n\n"
    text_typed += "Ich addiere\n"
    text_typed += "Und subtrahiere, \n\n"
    text_typed += "Kontrolliere\nUnd komponiere\nUnd wenn ich diese Taste drück,\nSpielt er ein kleines Musikstück?\n\n"

    text_typed += "Ich bin der Musikant mit Taschenrechner in der Hand!\n\n"
    text_typed += "Ich addiere\n"
    text_typed += "Und subtrahiere, \n\n"
    text_typed += "Kontrolliere\nUnd komponiere\nUnd wenn ich diese Taste drück,\nSpielt er ein kleines Musikstück?\n\n"

    center_x = width // 2
    center_y = height // 2

    font = load_font("data/miso-bold.ttf")
    # font = create_font("Arial")

    shape_space = load_shape("data/space.svg")  # GAP: load_shape() (SVG) is missing
    shape_space2 = load_shape("data/space2.svg")  # GAP: load_shape() (SVG) is missing
    shape_period = load_shape("data/period.svg")  # GAP: load_shape() (SVG) is missing
    shape_comma = load_shape("data/comma.svg")  # GAP: load_shape() (SVG) is missing
    shape_exclamationmark = load_shape("data/exclamationmark.svg")  # GAP: load_shape() (SVG) is missing
    shape_questionmark = load_shape("data/questionmark.svg")  # GAP: load_shape() (SVG) is missing
    shape_return = load_shape("data/return.svg")  # GAP: load_shape() (SVG) is missing

    cursor(HAND)


def draw():
    global center_x, center_y, save_png
    background(1.0)
    no_stroke()
    text_align(LEFT)

    if mouse_is_pressed:
        center_x = mouse_x - offset_x
        center_y = mouse_y - offset_y

    # allways produce the same sequence of random numbers
    random_seed(act_random_seed)

    translate(center_x, center_y)
    scale(zoom)

    for i in range(len(text_typed)):
        font_size = 25
        text_font(font)
        text_size(font_size)
        letter = text_typed[i]
        letter_width = text_width(letter)

        # ------ letter rule table ------
        if letter == " ":  # space
            # 50% left, 50% right
            dir = floor(random(0, 2))
            if dir == 0:
                shape(shape_space, 0, 0)  # GAP: shape(s, x, y)
                translate(1.9, 0)
                rotate(PI / 4)
            if dir == 1:
                shape(shape_space2, 0, 0)  # GAP: shape(s, x, y)
                translate(13, -5)
                rotate(-PI / 4)

        elif letter == ",":
            shape(shape_comma, 0, 0)  # GAP: shape(s, x, y)
            translate(34, 15)
            rotate(PI / 4)

        elif letter == ".":
            shape(shape_period, 0, 0)  # GAP: shape(s, x, y)
            translate(56, -54)
            rotate(-PI / 2)

        elif letter == "!":
            shape(shape_exclamationmark, 0, 0)  # GAP: shape(s, x, y)
            translate(42, -17.4)
            rotate(-PI / 4)

        elif letter == "?":
            shape(shape_questionmark, 0, 0)  # GAP: shape(s, x, y)
            translate(42, -18)
            rotate(-PI / 4)

        elif letter == "\n":  # return
            shape(shape_return, 0, 0)  # GAP: shape(s, x, y)
            translate(0, 10)
            rotate(PI)

        else:  # all others
            fill(0.0)
            text(letter, 0, 0)
            translate(letter_width, 0)

    # blink cursor after text
    fill(0.0)
    if frame_count // 6 % 2 == 0:
        rect(0, 0, 15, 2)

    if save_png:
        save_png = False
        save_frame(timestamp() + ".png")


def mouse_pressed():
    global offset_x, offset_y
    offset_x = mouse_x - center_x
    offset_y = mouse_y - center_y


def key_released():
    global save_png, act_random_seed
    if key_code == CONTROL:
        save_png = True
    if key_code == ALT:
        act_random_seed += 1
    print(act_random_seed)


def key_pressed():
    global text_typed, zoom
    if key is not None:
        if key_code == DELETE or key_code == BACKSPACE:
            text_typed = text_typed[0:max(0, len(text_typed) - 1)]
        elif key_code == TAB:
            pass
        elif key_code == ENTER:
            # enable linebreaks
            text_typed += "\n"
        elif key_code == ESCAPE:
            pass
        else:
            text_typed += key

    # zoom
    if key_code == UP:
        zoom += 0.05
    if key_code == DOWN:
        zoom -= 0.05


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
