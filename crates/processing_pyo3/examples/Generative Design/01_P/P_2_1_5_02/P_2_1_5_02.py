# P_2_1_5_02
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

# Place stacked circles of randomised heights at the mouse position
# to create a moire effect drawing
#
# MOUSE
# mouse              : place circle
#
# KEYS
# s                   : save png
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
from datetime import datetime

from processing_py import *


class Shape:
    def __init__(self, x, y, r):
        self.x = x
        self.y = y
        self.r = r

    def draw(self):
        i = 0
        while i < self.r:
            circle(self.x, self.y, i)
            i += density


shapes = []
min_radius = 5
max_radius = 250
density = 5


def setup():
    size(800, 800)
    no_fill()

    shapes.append(Shape(width / 2, height / 2, width))


def draw():
    background(1.0)

    for shape in shapes:
        shape.draw()


def mouse_released():
    shapes.append(Shape(mouse_x, mouse_y, random(min_radius, max_radius)))


def key_pressed():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
