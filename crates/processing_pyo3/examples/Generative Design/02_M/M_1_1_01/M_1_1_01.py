# M_1_1_01
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

# draws a random chart and shows how to use randomSeed.
#
# MOUSE
# click               : new random line
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *

act_random_seed = 42


def setup():
    size(1024, 256)


def draw():
    background(1.0)

    # line
    stroke(0.0, 0.51, 0.64)
    stroke_weight(1)
    stroke_join(ROUND)
    no_fill()

    random_seed(act_random_seed)
    begin_shape()
    for x in range(0, width, 10):
        y = random(0, height)
        vertex(x, y)
    end_shape()

    # dots
    no_stroke()
    fill(0.0)

    random_seed(act_random_seed)
    for x in range(0, width, 10):
        y = random(0, height)
        ellipse(x, y, 3, 3)


def mouse_released():
    global act_random_seed
    act_random_seed = int(random(100000))


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
