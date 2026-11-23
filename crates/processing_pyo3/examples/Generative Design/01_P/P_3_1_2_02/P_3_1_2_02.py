# P_3_1_2_02
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
# click + drag        : move canvas
#
# KEYS
# a-z                 : text input (keyboard)
# space               : random straight / small curve
# ,.!?                : curves
# :+-xz               : icons
# o                   : station with the last 7 typed letters as name
# a u                 : stop
# del, backspace      : remove last letter
# arrow up            : zoom canvas +
# arrow down          : zoom canvas -
# ctrl                : save png
from datetime import datetime

from processing_py import *

save_png = False

font = None
text_typed = "Was hier folgt ist Text! So asnt, und mag. Ich mag Text sehr."

shape_space = None
shape_space2 = None
shape_period = None
shape_comma = None
shape_exclamationmark = None
shape_questionmark = None
shape_return = None
icon1 = None
icon2 = None
icon3 = None
icon4 = None
icon5 = None

center_x = 0
center_y = 0
offset_x = 0
offset_y = 0
zoom = 0.75

palette = [
    color(253 / 255, 195 / 255, 0.0), color(0.0), color(0.0, 158 / 255, 224 / 255), color(99 / 255, 33 / 255, 129 / 255),
    color(121 / 255, 156 / 255, 19 / 255), color(226 / 255, 0.0, 26 / 255), color(224 / 255, 134 / 255, 178 / 255)]
act_color_index = 0


def setup():
    global center_x, center_y, font
    global shape_space, shape_space2, shape_period, shape_comma, shape_exclamationmark
    global shape_questionmark, shape_return, icon1, icon2, icon3, icon4, icon5
    size(800, 600)
    # make window resizable
    window_resizable(True)

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
    icon1 = load_shape("data/icon1.svg")  # GAP: load_shape() (SVG) is missing
    icon2 = load_shape("data/icon2.svg")  # GAP: load_shape() (SVG) is missing
    icon3 = load_shape("data/icon3.svg")  # GAP: load_shape() (SVG) is missing
    icon4 = load_shape("data/icon4.svg")  # GAP: load_shape() (SVG) is missing
    icon5 = load_shape("data/icon5.svg")  # GAP: load_shape() (SVG) is missing

    shape_space.disable_style()  # GAP: Shape.disable_style()
    shape_space2.disable_style()  # GAP: Shape.disable_style()
    shape_period.disable_style()  # GAP: Shape.disable_style()
    shape_comma.disable_style()  # GAP: Shape.disable_style()
    shape_exclamationmark.disable_style()  # GAP: Shape.disable_style()
    shape_questionmark.disable_style()  # GAP: Shape.disable_style()
    shape_return.disable_style()  # GAP: Shape.disable_style()

    cursor(HAND)


def draw():
    global center_x, center_y, act_color_index, save_png
    background(1.0)
    no_stroke()
    text_align(LEFT)

    if mouse_is_pressed:
        center_x = mouse_x - offset_x
        center_y = mouse_y - offset_y

    translate(center_x, center_y)
    scale(zoom)

    push_matrix()

    random_seed(0)

    act_color_index = 0
    fill(palette[act_color_index])
    rect(0, -25, 10, 35)

    for i in range(len(text_typed)):
        font_size = 25
        text_font(font)
        text_size(font_size)
        letter = text_typed[i]
        letter_width = text_width(letter)

        # ------ letter rule table ------
        if letter == " ":  # space
            # 60% noturn, 20% left, 20% right
            dir = floor(random(0, 5))
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
            # start a new line at a random position near the center
            rect(0, -25, 10, 35)
            pop_matrix()
            push_matrix()
            translate(random(-300, 300), random(-300, 300))
            rotate(floor(random(8)) * PI / 4)
            # choose nest color from the palette
            act_color_index = (act_color_index + 1) % len(palette)
            fill(palette[act_color_index])
            rect(0, -25, 10, 35)

        elif letter == "o":  # Station big
            rect(0, 0 - 15, letter_width + 1, 15)
            push_style()
            fill(0.0)
            station = text_typed[i - 10:i - 1]
            station = station.lower()
            station = station.replace(" ", "")
            station = station[0:1].upper() + station[1:len(station) - 1]
            text(station, -10, 40)

            ellipse(-5, -7, 33, 33)
            fill(1.0)
            ellipse(-5, -7, 25, 25)
            pop_style()
            translate(letter_width, 0)

        elif letter == "a":  # Station small left
            rect(0, 0 - 15, letter_width + 1, 25)
            rect(0, 0 - 15, letter_width + 1, 15)
            translate(letter_width, 0)

        elif letter == "u":  # Station small right
            rect(0, 0 - 25, letter_width + 1, 25)
            rect(0, 0 - 15, letter_width + 1, 15)
            translate(letter_width, 0)

        elif letter == ":":  # icon
            shape(icon1, 0, -60, 30, 30)  # GAP: shape(s, x, y, w, h)

        elif letter == "+":  # icon
            shape(icon2, 0, -60, 35, 30)  # GAP: shape(s, x, y, w, h)

        elif letter == "-":  # icon
            shape(icon3, 0, -60, 30, 30)  # GAP: shape(s, x, y, w, h)

        elif letter == "x":  # icon
            shape(icon4, 0, -60, 30, 30)  # GAP: shape(s, x, y, w, h)

        elif letter == "z":  # icon
            shape(icon5, 0, -60, 30, 30)  # GAP: shape(s, x, y, w, h)

        else:  # all others
            # text(letter, 0, 0)

            rect(0, 0 - 15, letter_width + 1, 15)
            translate(letter_width, 0)
            # rotate(-0.05)

    # blink cursor after text
    fill(0.78, 0.12, 0.16)
    if frame_count // 6 % 2 == 0:
        rect(0, 0, 15, 2)

    pop_matrix()

    if save_png:
        save_png = False
        save_frame(timestamp() + ".png")


def mouse_pressed():
    global offset_x, offset_y
    offset_x = mouse_x - center_x
    offset_y = mouse_y - center_y


def key_released():
    global save_png
    if key_code == CONTROL:
        save_png = True


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
