# P_2_1_5_04
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
# smooth path of any length, width, smoothness and colour.
#
# MOUSE
# mouse               : click and drag to draw moire pattern path
#
# KEYS
# 1                   : set color to red
# 2                   : set color to green
# 3                   : set color to blue
# 4                   : set color to black
# arrow right         : increase smoothness
# arrow left          : decrease smoothness
# arrow up            : increase rectangle width
# arrow down          : decrease rectangle width
# s                   : save png
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
from datetime import datetime

from processing_py import *


class Shape:
    def __init__(self, h, c):
        self.shape_path = []
        self.h = h
        self.c = c

    def add_pos(self, x, y):
        new_pos = Vec2(x, y)
        last_pos = self.get_last_pos()
        if len(self.shape_path) == 0 or (last_pos and new_pos.dist(last_pos) > smoothness):
            self.shape_path.append(new_pos)

    def get_last_pos(self):
        if len(self.shape_path) == 0:
            return None
        return self.shape_path[len(self.shape_path) - 1]

    def draw(self):
        stroke(self.c)
        i = -self.h / 2
        while i < self.h / 2:
            begin_shape()
            for index in range(len(self.shape_path)):
                pos = self.shape_path[index]
                if index > 0:
                    previous_pos = self.shape_path[index - 1]
                    a = atan2(previous_pos.y - pos.y, previous_pos.x - pos.x)
                    offset_pos = Vec2.from_angle(a)
                    offset_pos.add(0, i)
                    offset_pos = offset_pos.rotate(a)
                    offset_pos.add(pos)
                    curve_vertex(offset_pos.x, offset_pos.y)
            end_shape()
            i += density


shapes = []
density = 2.5
shape_height = 64
shape_color = None
smoothness = 0

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
        new_shape.h = shape_height
        new_shape.c = shape_color
        new_shape.add_pos(mouse_x, mouse_y)
        new_shape.draw()


def mouse_pressed():
    global new_shape
    new_shape = Shape(shape_height, shape_color)
    new_shape.add_pos(mouse_x, mouse_y)


def mouse_released():
    global new_shape
    shapes.append(new_shape)
    new_shape = None


def key_pressed():
    global shapes, shape_color, shape_height, smoothness
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

    if key == " ":
        shapes = []
        redraw()

    if key_code == RIGHT_ARROW:
        smoothness += density
    if key_code == LEFT_ARROW:
        smoothness -= density

    if key_code == UP:
        shape_height += density
    if key_code == DOWN:
        shape_height -= density


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
