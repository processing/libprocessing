# P_2_1_2_01
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

# changing size and position of circles in a grid
#
# MOUSE
# position x          : circle position
# position y          : circle size
# left click          : random position
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *

tile_count = 20
circle_color = color(0.0)
circle_alpha = 180
act_random_seed = 0


def setup():
    size(600, 600)


def draw():
    translate(width / tile_count / 2, height / tile_count / 2)

    background(1.0)
    no_fill()

    random_seed(act_random_seed)

    stroke(circle_color.with_alpha(circle_alpha / 255))
    stroke_weight(mouse_y // 60)

    for grid_y in range(tile_count):
        for grid_x in range(tile_count):
            pos_x = width / tile_count * grid_x
            pos_y = height / tile_count * grid_y

            shift_x = random(-mouse_x, mouse_x) / 20
            shift_y = random(-mouse_x, mouse_x) / 20

            ellipse(pos_x + shift_x, pos_y + shift_y, mouse_y // 15, mouse_y // 15)


def mouse_pressed():
    global act_random_seed
    act_random_seed = int(random(100000))


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
