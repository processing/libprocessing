# P_3_1_3_03
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
# drawing the letters frequency with lines and ellipses
#
# MOUSE
# position x          : random angle
# position y          : line length, size of ellipses, tracking
#
# KEYS
# 1                   : toggle alpha mode
# 2                   : toggle drawing of lines
# 3                   : toggle drawing of ellipses
# 4                   : toggle drawing of text
# s                   : save png
import os
from datetime import datetime

from processing_py import *

font = None
lines = []
joined_text = ""

alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÜß,.;:!? "
counters = [0] * len(alphabet)

pos_x = 0
pos_y = 0
tracking = 29

act_random_seed = 0

draw_alpha = True
draw_lines = True
draw_ellipses = True
draw_text = False


def setup():
    global lines, joined_text, font
    size(1400, 800)
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
    text_size(20)

    pos_x = 80
    pos_y = 300
    random_seed(act_random_seed)

    # ------ draw lines and ellipses ------
    for i in range(len(joined_text)):
        # again, find the index of the current letter in the alphabet
        s = joined_text[i].upper()
        uppercase_char = s[0]
        index = alphabet.find(uppercase_char)
        if index < 0:
            continue

        # ------ calculate parameters ------
        char_alpha = 100
        if draw_alpha:
            char_alpha = counters[index]

        my = remap(mouse_y, 50, height - 50, 0, 1)
        my = constrain(my, 0, 1)
        char_size = counters[index] * my * 3

        mx = remap(mouse_x, 50, width - 50, 0, 1)
        mx = constrain(mx, 0, 1)
        line_length = char_size
        line_angle = random(-PI, PI) * mx - HALF_PI
        new_pos_x = line_length * cos(line_angle)
        new_pos_y = line_length * sin(line_angle)

        # ------ draw elements ------
        push_matrix()
        translate(pos_x, pos_y)
        stroke(hsva(273, 0.73, 0.51, char_alpha / 100))
        if draw_lines:
            line(0, 0, new_pos_x, new_pos_y)
        no_stroke()
        fill(hsva(52, 1.0, 0.71, char_alpha / 100))
        if draw_ellipses:
            ellipse(0, 0, char_size / 10, char_size / 10)
        pop_matrix()

        pos_x = int(pos_x + text_width(joined_text[i]))
        if pos_x >= width - 200 and uppercase_char == " ":
            pos_y += int(tracking * my + 30)
            pos_x = 80

    # ------ draw letters ------
    if draw_text:
        pos_x = 80
        pos_y = 300
        random_seed(act_random_seed)

        for i in range(len(joined_text)):
            # again, find the index of the current letter in the alphabet
            s = joined_text[i].upper()
            uppercase_char = s[0]
            index = alphabet.find(uppercase_char)
            if index < 0:
                continue

            # ------ calculate parameters ------
            char_alpha = 100
            if draw_alpha:
                char_alpha = counters[index]

            my = remap(mouse_y, 50, height - 50, 0, 1)
            my = constrain(my, 0, 1)
            char_size = counters[index] * my * 3

            mx = remap(mouse_x, 50, width - 50, 0, 1)
            mx = constrain(mx, 0, 1)
            line_length = char_size
            line_angle = random(-PI, PI) * mx - HALF_PI
            new_pos_x = line_length * cos(line_angle)
            new_pos_y = line_length * sin(line_angle)

            # ------ draw elements ------
            push_matrix()
            translate(pos_x, pos_y)
            fill(0.0, char_alpha / 100)
            text(joined_text[i], new_pos_x, new_pos_y)
            pop_matrix()

            pos_x = int(pos_x + text_width(joined_text[i]))
            if pos_x >= width - 200 and uppercase_char == " ":
                pos_y += int(tracking * my + 30)
                pos_x = 80


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


def mouse_pressed():
    global act_random_seed
    act_random_seed = int(random(100000))


def key_released():
    global draw_alpha, draw_lines, draw_ellipses, draw_text
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        draw_alpha = not draw_alpha
    if key == "2":
        draw_lines = not draw_lines
    if key == "3":
        draw_ellipses = not draw_ellipses
    if key == "4":
        draw_text = not draw_text


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
