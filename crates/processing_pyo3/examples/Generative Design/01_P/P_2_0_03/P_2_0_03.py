# P_2_0_03
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

# drawing with a changing shape by draging the mouse.
#
# MOUSE
# position x          : length
# position y          : thickness and number of lines
# drag                : draw
#
# KEYS
# 1-3                 : stroke color
# del, backspace      : erase
# s                   : save png
from datetime import datetime

from processing_py import *

stroke_color = color(0.0, 10 / 255)


def setup():
    size(720, 720)
    no_fill()
    background(1.0)


def draw():
    if mouse_is_pressed:
        push_matrix()
        translate(width / 2, height / 2)

        circle_resolution = int(remap(mouse_y + 100, 0, height, 2, 10))
        radius = mouse_x - width / 2
        angle = TWO_PI / circle_resolution

        stroke_weight(2)
        stroke(stroke_color)

        begin_shape()
        for i in range(circle_resolution + 1):
            x = 0 + cos(angle * i) * radius
            y = 0 + sin(angle * i) * radius
            vertex(x, y)
        end_shape()

        pop_matrix()


def key_released():
    global stroke_color
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        stroke_color = color(0.0, 0.1)
    elif key == "2":
        stroke_color = hsva(192, 1.0, 0.64, 0.1)
    elif key == "3":
        stroke_color = hsva(52, 1.0, 0.71, 0.1)


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
