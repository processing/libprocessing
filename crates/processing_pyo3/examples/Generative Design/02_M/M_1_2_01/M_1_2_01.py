# M_1_2_01
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

# order vs random!
# how to interpolate beetween a free composition (random) and a circle shape (order)
#
# MOUSE
# position x          : fade between random and circle shape
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *

act_random_seed = 0
count = 150


def setup():
    size(800, 800)
    cursor(CROSS)


def draw():
    background(1.0)
    no_stroke()

    fader_x = mouse_x / width

    random_seed(act_random_seed)
    angle = radians(360 / count)
    for i in range(count):
        # positions
        random_x = random(0, width)
        random_y = random(0, height)
        circle_x = width / 2 + cos(angle * i) * 300
        circle_y = height / 2 + sin(angle * i) * 300

        x = lerp(random_x, circle_x, fader_x)
        y = lerp(random_y, circle_y, fader_x)

        fill(0.0, 0.51, 0.64)
        ellipse(x, y, 11, 11)


def mouse_released():
    global act_random_seed
    act_random_seed = int(random(100000))


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
