# P_3_2_4_01
#
# Generative Gestaltung – Creative Coding im Web
# ISBN: 978-3-87439-902-9, First Edition, Hermann Schmidt, Mainz, 2018
# Benedikt Groß, Hartmut Bohnacker, Julia Laub, Claudius Lazzeroni
# with contributions by Joey Lee and Niels Poldervaart
# Copyright 2018
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
# Ported from the p5.js original to processing_py.

# Drawing tool for creating moire effect compositions using
# smooth path of any length, width, smoothness and colour.
#
# MOUSE
# position x          : path simplification
# position y          : ribbon width
#
# KEYS
# arrow right         : increase path density
# arrow left          : decrease path density
# arrow up            : increase font size
# arrow down          : decrease font size
# control             : save png
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
from datetime import datetime

from processing_py import *


class Letter:
    def __init__(self, char, x, y):
        self.char = char
        self.x = x
        self.y = y

    def draw(self):
        text_size(font_size)
        path = text_to_points(self.char, self.x, self.y, path_sample_factor)
        stroke(shape_color)

        d = 0
        while d < ribbon_width:
            begin_shape()

            for i in range(len(path)):
                pos = path[i]

                if i + 1 < len(path):
                    next_pos = path[i + 1]
                    p0 = Vec2(pos[0], pos[1])
                    p1 = Vec2(next_pos[0], next_pos[1])
                    v = p1 - p0
                    v = v.normalize()
                    v = v.rotate(HALF_PI)
                    v.mult(d)
                    pneu = p0 + v
                    curve_vertex(pneu.x, pneu.y)

            end_shape(CLOSE)
            d += density


letters = []
density = 2.5
ribbon_width = 92
shape_color = None
font_size = 800
path_simplification = 0
path_sample_factor = 0.1

text_typed = "a"

font = None


def setup():
    global font, shape_color
    full_screen()
    font = load_font("data/NotoSans-Bold.ttf")
    text_font(font)
    no_fill()
    stroke_weight(1)
    shape_color = color(0.0)

    create_letters()


def draw():
    global path_sample_factor, ribbon_width
    background(1.0)

    translate(100, height * 0.75)

    path_sample_factor = 0.1 * pow(0.02, mouse_x / width)
    ribbon_width = remap(mouse_y, 0, height, 1, 200)

    for i in range(len(letters)):
        letters[i].draw()


def create_letters():
    global letters
    letters = []
    chars = list(text_typed)

    text_size(font_size)
    x = 0
    for i in range(len(chars)):
        if i > 0:
            chars_before = text_typed[0:i]
            x = text_bounds(chars_before, 0, 0)[2]
        new_letter = Letter(chars[i], x, 0)
        letters.append(new_letter)


def key_released():
    global density, font_size
    if key_code == CONTROL:
        save_frame(timestamp() + ".png")

    if key_code == LEFT_ARROW:
        density *= 1.25
    if key_code == RIGHT_ARROW:
        density /= 1.25

    if key_code == UP:
        font_size *= 1.1
        create_letters()
    if key_code == DOWN:
        font_size /= 1.1
        create_letters()

    if key_code == ENTER:
        create_letters()


def key_pressed():
    global text_typed
    if key_code == DELETE or key_code == BACKSPACE:
        if len(text_typed) > 0:
            text_typed = text_typed[0 : len(text_typed) - 1]
            create_letters()


def key_typed():
    global text_typed
    if key is not None and ord(key) >= 32:
        text_typed += key
        create_letters()


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
