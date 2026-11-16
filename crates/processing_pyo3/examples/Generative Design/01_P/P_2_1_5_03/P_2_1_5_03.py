# P_2_1_5_03
#
# Generative Gestaltung – Creative Coding im Web
# ISBN: 978-3-87439-902-9, First Edition, Hermann Schmidt, Mainz, 2018
# Benedikt Groß, Hartmut Bohnacker, Julia Laub, Claudius Lazzeroni
# with contributions by Joey Lee and Niels Poldervaart
# Copyright 2018
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
# Ported from the p5.js original to processing_py.

# Drawing tool for creating moire effect compositions using
# rectangles of various widths, lengths, angles and colours.
#
# MOUSE
# mouse               : place moire effect rectangle
#
# KEYS
# 1                   : set color to red
# 2                   : set color to green
# 3                   : set color to blue
# 4                   : set color to black
# arrow up            : increase rectangle width
# arrow down          : decrease rectangle width
# s                   : save png
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
from datetime import datetime

from processing_py import *


class Shape:
    def __init__(self, x1, y1, x2, y2, h, c):
        self.x1 = x1
        self.y1 = y1
        self.x2 = x2
        self.y2 = y2
        self.h = h
        self.c = c

    def draw(self):
        w = dist(self.x1, self.y1, self.x2, self.y2)
        a = atan2(self.y2 - self.y1, self.x2 - self.x1)
        stroke(self.c)
        push()
        translate(self.x1, self.y1)
        rotate(a)
        translate(0, -self.h / 2)
        i = 0
        while i < self.h:
            line(0, i, w, i)
            i += density
        pop()


shapes = []
density = 2.5
shape_height = 64
shape_color = None

new_shape = None


def setup():
    global shape_color
    size(800, 800)
    no_fill()
    shape_color = color(0.0)


def draw():
    background(1.0)

    for shape in shapes:
        shape.draw()

    if new_shape:
        new_shape.x2 = mouse_x
        new_shape.y2 = mouse_y
        new_shape.h = shape_height
        new_shape.c = shape_color
        new_shape.draw()


def mouse_pressed():
    global new_shape
    new_shape = Shape(pmouse_x, pmouse_y, mouse_x, mouse_y, shape_height, shape_color)


def mouse_released():
    global new_shape
    shapes.append(new_shape)
    new_shape = None


def key_pressed():
    global shape_color, shape_height
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        shape_color = color(1.0, 0.0, 0.0)
    if key == "2":
        shape_color = color(0.0, 1.0, 0.0)
    if key == "3":
        shape_color = color(0.0, 0.0, 1.0)
    if key == "4":
        shape_color = color(0.0)

    if key_code == UP:
        shape_height += density
    if key_code == DOWN:
        shape_height -= density


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
