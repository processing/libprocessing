# P_2_3_3_01
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

# draw tool. shows how to draw with dynamic elements.
#
# MOUSE
# drag                : draw with text
#
# KEYS
# del, backspace      : clear screen
# arrow up            : angle distortion +
# arrow down          : angle distortion -
# s                   : save png
from datetime import datetime

from processing_py import *

x = 0
y = 0
step_size = 5.0

font = None
letters = "Sie hören nicht die folgenden Gesänge, Die Seelen, denen ich die ersten sang, Zerstoben ist das freundliche Gedränge, Verklungen ach! der erste Wiederklang."
font_size_min = 3
angle_distortion = 0.0

counter = 0


def setup():
    global x, y, font
    # use full screen size
    size(display_width, display_height)  # GAP: display_width / display_height are 0 until the first draw(), so the window cannot be created
    background(1.0)
    cursor(CROSS)

    x = mouse_x
    y = mouse_y

    font = create_font("American Typewriter")
    # font = create_font("ArnhemFineTT-Normal")
    text_font(font)
    text_size(font_size_min)
    text_align(LEFT)
    fill(0.0)

    # # load an image in background
    # img = load_image(select_input("select a background image"))
    # image(img, 0, 0, width, height)


def draw():
    global x, y, step_size, counter
    if mouse_is_pressed:
        d = dist(x, y, mouse_x, mouse_y)
        text_font(font)
        text_size(font_size_min + d / 2)
        new_letter = letters[counter]
        step_size = text_width(new_letter)

        if d > step_size:
            angle = atan2(mouse_y - y, mouse_x - x)

            push_matrix()
            translate(x, y)
            rotate(angle + random(angle_distortion))
            text(new_letter, 0, 0)
            pop_matrix()

            counter += 1
            if counter > len(letters) - 1:
                counter = 0

            x = x + cos(angle) * step_size
            y = y + sin(angle) * step_size


def mouse_pressed():
    global x, y
    x = mouse_x
    y = mouse_y


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)


def key_pressed():
    global angle_distortion
    # angle_distortion ctrls arrowkeys up/down
    if key_code == UP:
        angle_distortion += 0.1
    if key_code == DOWN:
        angle_distortion -= 0.1


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
