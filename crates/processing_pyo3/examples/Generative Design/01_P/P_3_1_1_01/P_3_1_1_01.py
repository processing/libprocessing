# P_3_1_1_01
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

# typewriter. time reactive.
#
# MOUSE
# position y           : adjust spacing (line height)
#
# KEYS
# a-z                  : text input (keyboard)
# backspace            : delete last typed letter
# ctrl                 : save png
from datetime import datetime

from processing_py import *

do_save = False

text_typed = "Type slow and fast!"
font_sizes = [0.0] * len(text_typed)
min_font_size = 15
max_font_size = 800
new_font_size = 0

p_millis = 0
max_time_delta = 5000.0

spacing = 2  # line height
tracking = 0  # between letters
font = None


def setup():
    global font, p_millis
    size(800, 600)
    # make window resizable
    window_resizable(True)

    font = create_font("Arial")

    no_cursor()

    # init font_sizes
    for i in range(len(text_typed)):
        font_sizes[i] = min_font_size

    p_millis = millis()


def draw():
    global spacing, new_font_size, do_save
    background(1.0)
    text_align(LEFT)
    fill(0.0)
    no_stroke()

    spacing = remap(mouse_y, 0, height, 0, 120)
    translate(0, 200 + spacing)

    x = 0
    y = 0
    font_size = 20

    for i in range(len(text_typed)):
        # get fontsize for the actual letter from the list
        font_size = font_sizes[i]
        text_font(font)
        text_size(font_size)
        letter = text_typed[i]
        letter_width = text_width(letter) + tracking

        if x + letter_width > width:
            # start new line and add line height
            x = 0
            y += spacing

        # draw letter
        text(letter, x, y)
        # update x-coordinate for next letter
        x += letter_width

    # blinking cursor after text
    time_delta = millis() - p_millis
    new_font_size = remap(time_delta, 0, max_time_delta, min_font_size, max_font_size)
    new_font_size = min(new_font_size, max_font_size)

    fill(0.78, 0.12, 0.16)
    if frame_count // 10 % 2 == 0:
        fill(1.0)
    rect(x, y, new_font_size / 2, new_font_size / 20)

    if do_save:
        do_save = False
        save_frame(timestamp() + "_##.png")


def key_released():
    global do_save
    # export png
    if key_code == CONTROL:
        do_save = True


def key_pressed():
    global text_typed, p_millis
    if key is not None:
        if key_code == DELETE or key_code == BACKSPACE:
            if len(text_typed) > 0:
                text_typed = text_typed[0:max(0, len(text_typed) - 1)]
                font_sizes.pop()
        elif key_code == TAB or key_code == ENTER or key_code == ESCAPE:
            # disable those keys
            pass
        else:
            text_typed += key
            font_sizes.append(new_font_size)

        # reset timer
        p_millis = millis()


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
