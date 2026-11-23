# P_2_3_1_01
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

# draw tool. draw with a rotating line.
#
# MOUSE
# drag                : draw
#
# KEYS
# 1-4                 : switch default colors
# del, backspace      : clear screen
# d                   : reverse direction and mirrow angle
# space               : new random color
# arrow left          : rotaion speed -
# arrow right         : rotaion speed +
# arrow up            : line length +
# arrow down          : line length -
# s                   : save png
from datetime import datetime

from processing_py import *

col = color(181 / 255, 157 / 255, 0.0, 100 / 255)
line_length = 0
angle = 0
angle_speed = 1.0


def setup():
    # use full screen size
    size(display_width, display_height)  # GAP: display_width / display_height are 0 until the first draw(), so the window cannot be created
    background(1.0)
    cursor(CROSS)


def draw():
    global angle
    if mouse_is_pressed:
        push_matrix()
        stroke_weight(1.0)
        no_fill()
        stroke(col)
        translate(mouse_x, mouse_y)
        rotate(radians(angle))
        line(0, 0, line_length, 0)
        pop_matrix()

        angle += angle_speed


def mouse_pressed():
    global line_length
    # create a new random line length
    line_length = random(70, 200)


def key_released():
    global angle, angle_speed, col
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


def key_pressed():
    global line_length, angle_speed
    if key_code == UP:
        line_length += 5
    if key_code == DOWN:
        line_length -= 5
    if key_code == LEFT_ARROW:
        angle_speed -= 0.5
    if key_code == RIGHT_ARROW:
        angle_speed += 0.5


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
