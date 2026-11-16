# P_2_0_01
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

# drawing a filled circle with lines.
#
# MOUSE
# position x          : length
# position y          : thickness and number of lines
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *


def setup():
    size(550, 550)


def draw():
    stroke_cap(SQUARE)
    no_fill()
    background(1.0)
    translate(width / 2, height / 2)

    circle_resolution = int(remap(mouse_y, 0, height, 2, 80))
    radius = mouse_x - width / 2 + 0.5
    angle = TWO_PI / circle_resolution

    stroke_weight(mouse_y / 20)

    begin_shape()
    for i in range(circle_resolution + 1):
        x = cos(angle * i) * radius
        y = sin(angle * i) * radius
        line(0, 0, x, y)
        # vertex(x, y)
    end_shape()


def key_pressed():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
