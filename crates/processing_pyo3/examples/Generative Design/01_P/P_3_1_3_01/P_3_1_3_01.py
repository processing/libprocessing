# P_3_1_3_01
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
# changing the letters alpha value in relation to frequency
#
# MOUSE
# position x          : interpolate between normal text and sorted position
#
# KEYS
# 1                   : toggle alpha mode
# s                   : save png
import os
from datetime import datetime

from processing_py import *

font = None
joined_text = ""

alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÜß,.;:!? "
counters = [0] * len(alphabet)

char_size = 0.0
char_color = 0
pos_x = 0
pos_y = 0

draw_alpha = True


def setup():
    global joined_text, font
    size(670, 800)
    # open() resolves against the cwd, so build the path from this file
    path = os.path.join(os.path.dirname(__file__), "data", "faust_kurz.txt")
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines()
    joined_text = " ".join(lines)

    font = create_font("Courier")

    count_characters()


def draw():
    global pos_x, pos_y
    text_font(font)
    background(1.0)
    no_stroke()

    pos_x = 20
    pos_y = 40

    # go through all characters in the text to draw them
    for i in range(len(joined_text)):
        # again, find the index of the current letter in the alphabet
        s = joined_text[i].upper()
        uppercase_char = s[0]
        index = alphabet.find(uppercase_char)
        if index < 0:
            continue

        if draw_alpha:
            fill(0.34, 0.14, 0.51, counters[index] * 3 / 255)
        else:
            fill(0.34, 0.14, 0.51)
        text_size(18)

        sort_y = index * 20 + 40
        m = remap(mouse_x, 50, width - 50, 0, 1)
        m = constrain(m, 0, 1)
        inter_y = lerp(pos_y, sort_y, m)

        text(joined_text[i], pos_x, inter_y)

        pos_x = int(pos_x + text_width(joined_text[i]))
        if pos_x >= width - 200 and uppercase_char == " ":
            pos_y += 30
            pos_x = 20


def count_characters():
    for i in range(len(joined_text)):
        # get one char from the text and turn it to uppercase
        c = joined_text[i]
        s = c.upper()
        # take the first char of the result
        uppercase_char = s[0]
        # get the position of this char inside the alphabet string
        index = alphabet.find(uppercase_char)
        # increase the respective counter
        if index >= 0:
            counters[index] += 1


def key_released():
    global draw_alpha
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        draw_alpha = not draw_alpha


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
