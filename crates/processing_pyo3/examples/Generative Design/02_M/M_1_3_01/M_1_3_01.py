# M_1_3_01
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

# draws a chart based on noise values.
#
# MOUSE
# position x          : specify noise input range
# click               : new noise line
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *


def setup():
    size(1024, 256)


def draw():
    background(1.0)

    # line
    stroke(0.0, 0.51, 0.64)
    stroke_weight(1)
    stroke_join(ROUND)
    no_fill()

    noise_x_range = int(mouse_x / 10)
    print("noiseXRange: 0-" + str(noise_x_range))

    begin_shape()
    for x in range(0, width, 10):
        noise_x = remap(x, 0, width, 0, noise_x_range)
        y = noise(noise_x) * height
        vertex(x, y)
    end_shape()

    # dots
    no_stroke()
    fill(0.0)

    for x in range(0, width, 10):
        noise_x = remap(x, 0, width, 0, noise_x_range)
        y = noise(noise_x) * height
        ellipse(x, y, 3, 3)


def mouse_released():
    noise_seed(int(random(100000)))


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
