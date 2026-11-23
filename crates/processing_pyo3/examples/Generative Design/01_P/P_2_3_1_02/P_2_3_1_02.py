# P_2_3_1_02
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

# draw tool. draw with a rotating element (svg file).
#
# MOUSE
# drag                : draw
#
# KEYS
# 1-4                 : switch default colors
# 5-9                 : switch brush element
# del, backspace      : clear screen
# d                   : reverse direction and mirrow angle
# space               : new random color
# arrow left          : rotaion speed -
# arrow right         : rotaion speed +
# arrow up            : module size +
# arrow down          : module size -
# shift               : limit drawing direction
# s                   : save png
from datetime import datetime

from processing_py import *

col = color(181 / 255, 157 / 255, 0.0, 100 / 255)
line_module_size = 0
angle = 0
angle_speed = 1.0
line_module = None

click_pos_x = 0
click_pos_y = 0


def setup():
    # use full screen size
    size(display_width, display_height)  # GAP: display_width / display_height are 0 until the first draw(), so the window cannot be created
    background(1.0)
    cursor(CROSS)


def draw():
    global angle
    if mouse_is_pressed:
        x = mouse_x
        y = mouse_y
        if key_is_pressed and key_code == SHIFT:
            if abs(click_pos_x - x) > abs(click_pos_y - y):
                y = click_pos_y
            else:
                x = click_pos_x

        stroke_weight(0.75)
        no_fill()
        stroke(col)
        push_matrix()
        translate(x, y)
        rotate(radians(angle))
        if line_module is not None:
            shape(line_module, 0, 0, line_module_size, line_module_size)  # GAP: shape(s, x, y, w, h)
        else:
            line(0, 0, line_module_size, line_module_size)
        angle = angle + angle_speed
        pop_matrix()


def mouse_pressed():
    global line_module_size, click_pos_x, click_pos_y
    # create a new random color and line length
    line_module_size = random(50, 160)

    # remember click position
    click_pos_x = mouse_x
    click_pos_y = mouse_y


def key_released():
    global angle, angle_speed, col, line_module
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    # reverse direction and mirrow angle
    if key == "d" or key == "D":
        angle = angle + 180
        angle_speed = angle_speed * -1

    # r g b alpha
    if key == " ":
        col = color(random(1.0), random(1.0), random(1.0), random(80 / 255, 150 / 255))

    # default colors from 1 to 4
    if key == "1":
        col = color(181 / 255, 157 / 255, 0.0, 100 / 255)
    if key == "2":
        col = color(0.0, 130 / 255, 164 / 255, 100 / 255)
    if key == "3":
        col = color(87 / 255, 35 / 255, 129 / 255, 100 / 255)
    if key == "4":
        col = color(197 / 255, 0.0, 123 / 255, 100 / 255)

    # load svg for line module
    if key == "5":
        line_module = None
    if key == "6":
        line_module = load_shape("data/02.svg")  # GAP: load_shape() (SVG) is missing
    if key == "7":
        line_module = load_shape("data/03.svg")  # GAP: load_shape() (SVG) is missing
    if key == "8":
        line_module = load_shape("data/04.svg")  # GAP: load_shape() (SVG) is missing
    if key == "9":
        line_module = load_shape("data/05.svg")  # GAP: load_shape() (SVG) is missing
    if line_module is not None:
        line_module.disable_style()  # GAP: Shape.disable_style()


def key_pressed():
    global line_module_size, angle_speed
    if key_code == UP:
        line_module_size += 5
    if key_code == DOWN:
        line_module_size -= 5
    if key_code == LEFT_ARROW:
        angle_speed -= 0.5
    if key_code == RIGHT_ARROW:
        angle_speed += 0.5


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
