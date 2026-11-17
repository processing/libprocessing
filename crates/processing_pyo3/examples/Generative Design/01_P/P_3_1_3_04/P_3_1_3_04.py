# P_3_1_3_04
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

# analysing and sorting the letters of a text
# connecting subsequent letters with lines
#
# MOUSE
# position x          : interpolate between normal text and sorted position
#
# KEYS
# 1                   : toggle grey lines on/off
# 2                   : toggle colored lines on/off
# 3                   : toggle text on/off
# 4                   : switch all letters off
# 5                   : switch all letters on
# a-z                 : switch letter on/off
# ctrl                : save png
import os
from datetime import datetime

from processing_py import *

font = None
lines = []
joined_text = ""

alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÜß,.;:!? "
counters = [0] * len(alphabet)
draw_letters = [False] * len(alphabet)

char_size = 0.0
char_color = 0
pos_x = 20
pos_y = 50

draw_grey_lines = False
draw_colored_lines = True
draw_text = True


def setup():
    global lines, joined_text, font
    size(1200, 800)
    # open() resolves against the cwd, so build the path from this file
    path = os.path.join(os.path.dirname(__file__), "data", "faust_kurz.txt")
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines()  # load the text to analyse
    joined_text = " ".join(lines)

    font = create_font("Courier")

    for i in range(len(alphabet)):
        draw_letters[i] = True

    count_characters()


def draw():
    global pos_x, pos_y
    text_font(font)
    background(1.0)
    no_stroke()
    fill(0.0)

    translate(50, 0)

    pos_x = 0
    pos_y = 200
    sort_positions_x = [0.0] * len(alphabet)
    old_positions_x = [0.0] * len(alphabet)
    old_positions_y = [0.0] * len(alphabet)
    old_x = 0
    old_y = 0

    # draw counters
    if mouse_x >= width - 50:
        text_size(10)
        for i in range(len(alphabet)):
            text_align(LEFT)
            text(alphabet[i], -15, i * 20 + 40)
            text_align(RIGHT)
            text(str(counters[i]), -20, i * 20 + 40)

    # go through all characters in the text to draw them
    text_align(LEFT)
    text_size(18)

    for i in range(len(joined_text)):
        # again, find the index of the current letter in the alphabet
        s = joined_text[i].upper()
        uppercase_char = s[0]
        index = alphabet.find(uppercase_char)
        if index < 0:
            continue

        m = remap(mouse_x, 50, width - 50, 0, 1)
        m = constrain(m, 0, 1)

        sort_x = sort_positions_x[index]
        inter_x = lerp(pos_x, sort_x, m)

        sort_y = index * 20 + 40
        inter_y = lerp(pos_y, sort_y, m)

        if draw_letters[index]:
            if draw_grey_lines:
                if old_x != 0 and old_y != 0:
                    stroke(0.0, 0.1)
                    line(old_x, old_y, inter_x, inter_y)
                old_x = inter_x
                old_y = inter_y

            if draw_colored_lines:
                if old_positions_x[index] != 0 and old_positions_y[index] != 0:
                    stroke(hsva(index * 10, 0.8, 0.6, 0.5))
                    line(old_positions_x[index], old_positions_y[index], inter_x, inter_y)
                old_positions_x[index] = inter_x
                old_positions_y[index] = inter_y

            if draw_text:
                text(joined_text[i], inter_x, inter_y)
        else:
            old_x = 0
            old_y = 0

        sort_positions_x[index] += text_width(joined_text[i])
        pos_x = int(pos_x + text_width(joined_text[i]))
        if pos_x >= width - 200 and uppercase_char == " ":
            pos_y += 40
            pos_x = 0


def count_characters():
    for i in range(len(joined_text)):
        # get one char from the text and turn it to uppercase
        s = joined_text[i].upper()
        # take the first char of the result
        uppercase_char = s[0]
        # get the position of this char inside the alphabet string
        index = alphabet.find(uppercase_char)
        # increase the respective counter
        if index >= 0:
            counters[index] += 1


def key_released():
    global draw_grey_lines, draw_colored_lines, draw_text
    if key_code == CONTROL:
        save_frame(timestamp() + "_##.png")

    if key == "1":
        draw_grey_lines = not draw_grey_lines
    if key == "2":
        draw_colored_lines = not draw_colored_lines
    if key == "3":
        draw_text = not draw_text

    if key == "4":
        for i in range(len(alphabet)):
            draw_letters[i] = False
    if key == "5":
        for i in range(len(alphabet)):
            draw_letters[i] = True
    if key is not None:
        s = key.upper()
        uppercase_key = s[0]
        index = alphabet.find(uppercase_key)
        if index >= 0:
            draw_letters[index] = not draw_letters[index]


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
