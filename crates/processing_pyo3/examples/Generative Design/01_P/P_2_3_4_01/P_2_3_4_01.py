# P_2_3_4_01
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
# drag                : draw
#
# KEYS
# 1-9                 : switch module
# del, backspace      : clear screen
# arrow up            : module size +
# arrow down          : module size -
# arrow left          : step size -
# arrow right         : step size +
# s                   : save png
from datetime import datetime

from processing_py import *

x = 0
y = 0
step_size = 5.0
module_size = 25

line_module = None


def setup():
    global x, y, line_module
    # use full screen size
    size(display_width, display_height)  # GAP: display_width / display_height are 0 until the first draw(), so the window cannot be created
    background(1.0)
    x = mouse_x
    y = mouse_y
    cursor(CROSS)

    line_module = load_shape("data/01.svg")  # GAP: load_shape() (SVG) is missing

    # # load an image in background
    # img = load_image(select_input("select a background image"))
    # image(img, 0, 0, width, height)


def draw():
    global x, y
    if mouse_is_pressed:
        d = dist(x, y, mouse_x, mouse_y)

        if d > step_size:
            angle = atan2(mouse_y - y, mouse_x - x)

            push_matrix()
            translate(mouse_x, mouse_y)
            rotate(angle + PI)
            shape(line_module, 0, 0, d, module_size)  # GAP: shape(s, x, y, w, h)
            pop_matrix()

            x = x + cos(angle) * step_size
            y = y + sin(angle) * step_size


def mouse_pressed():
    global x, y
    x = mouse_x
    y = mouse_y


def key_released():
    global line_module
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)

    # load svg for line module
    if key == "1":
        line_module = load_shape("data/01.svg")  # GAP: load_shape() (SVG) is missing
    if key == "2":
        line_module = load_shape("data/02.svg")  # GAP: load_shape() (SVG) is missing
    if key == "3":
        line_module = load_shape("data/03.svg")  # GAP: load_shape() (SVG) is missing
    if key == "4":
        line_module = load_shape("data/04.svg")  # GAP: load_shape() (SVG) is missing
    if key == "5":
        line_module = load_shape("data/05.svg")  # GAP: load_shape() (SVG) is missing
    if key == "6":
        line_module = load_shape("data/06.svg")  # GAP: load_shape() (SVG) is missing
    if key == "7":
        line_module = load_shape("data/07.svg")  # GAP: load_shape() (SVG) is missing
    if key == "8":
        line_module = load_shape("data/08.svg")  # GAP: load_shape() (SVG) is missing
    if key == "9":
        line_module = load_shape("data/09.svg")  # GAP: load_shape() (SVG) is missing


def key_pressed():
    global module_size, step_size
    if key_code == UP or key_code == DOWN or key_code == LEFT_ARROW or key_code == RIGHT_ARROW:
        # module_size ctrls arrowkeys up/down
        if key_code == UP:
            module_size += 5
        if key_code == DOWN:
            module_size -= 5
        # step_size ctrls arrowkeys left/right
        step_size = max(step_size, 0.5)
        if key_code == LEFT_ARROW:
            step_size -= 0.5
        if key_code == RIGHT_ARROW:
            step_size += 0.5
        print("module_size: " + str(module_size) + "  step_size: " + str(step_size))


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
