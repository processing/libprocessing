# P_2_1_5_01
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

# Simple moire effect demonstration by moving, rotating
# and scaling a shape of densely packed lines over
# a background also consisting of densely packed lines.
#
# MOUSE
# mouseX              : overlay rotation or position x
# mouseY              : overlay scaling
#
# KEYS
# 1-2                 : switch draw mode
# s                   : save png
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
from datetime import datetime

from processing_py import *

draw_mode = 1


def setup():
    size(800, 800)
    no_fill()


def draw():
    background(1.0)

    translate(width / 2, height / 2)

    # first shape (fixed)
    stroke_weight(3)
    overlay()

    # second shape (dynamically translated/rotated and scaled)
    x = remap(mouse_x, 0, width, -50, 50)
    a = remap(mouse_x, 0, width, -0.5, 0.5)
    s = remap(mouse_y, 0, height, 0.7, 1)

    if draw_mode == 1:
        rotate(a)
    if draw_mode == 2:
        translate(x, 0)
    scale(s)

    stroke_weight(2)
    overlay()


def overlay():
    w = width - 100
    h = height - 100

    if draw_mode == 1:
        i = -w / 2
        while i < w / 2:
            line(i, -h / 2, i, h / 2)
            i += 5
    elif draw_mode == 2:
        i = 0
        while i < w:
            circle(0, 0, i)
            i += 10


def key_pressed():
    global draw_mode
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    # change draw mode
    if key == "1":
        draw_mode = 1
    if key == "2":
        draw_mode = 2


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
