# P_2_3_2_01
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

# draw tool. shows how to work with relations between elements.
#
# MOUSE
# drag                : draw
#
# KEYS
# 1                   : draw mode 1 - fixed distance
# 2                   : draw mode 2 - distance threshold
# del, backspace      : clear screen
# arrow up            : line length +
# arrow down          : line length -
# s                   : save png
from datetime import datetime

from processing_py import *

draw_mode = 1

col = color(random(1.0), random(1.0), random(1.0), random(100 / 255))
x = 0
y = 0
# new_x = 0
# new_y = 0
step_size = 5.0
line_length = 25


def setup():
    global x, y
    # use full screen size
    size(display_width, display_height)  # GAP: display_width / display_height are 0 until the first draw(), so the window cannot be created
    background(1.0)
    x = mouse_x
    y = mouse_y
    cursor(CROSS)


def draw():
    global x, y
    if mouse_is_pressed:
        d = dist(x, y, mouse_x, mouse_y)

        if d > step_size:
            angle = atan2(mouse_y - y, mouse_x - x)

            push_matrix()
            translate(x, y)
            rotate(angle)
            stroke(col)
            if frame_count % 2 == 0:
                stroke(150 / 255)
            line(0, 0, 0, line_length * random(0.95, 1.0) * d / 10)
            pop_matrix()

            if draw_mode == 1:
                x = x + cos(angle) * step_size
                y = y + sin(angle) * step_size
            else:
                x = mouse_x
                y = mouse_y


def mouse_pressed():
    global x, y, col
    x = mouse_x
    y = mouse_y
    col = color(random(1.0), random(1.0), random(1.0), random(100 / 255))
    # line_length = random(15, 50)


def key_released():
    global draw_mode
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)

    if key == "1":
        draw_mode = 1
    if key == "2":
        draw_mode = 2


def key_pressed():
    global line_length
    # line_length ctrls arrowkeys up/down
    if key_code == UP:
        line_length += 5
    if key_code == DOWN:
        line_length -= 5


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
