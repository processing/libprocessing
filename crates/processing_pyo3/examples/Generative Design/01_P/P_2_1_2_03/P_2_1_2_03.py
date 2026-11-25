# P_2_1_2_03
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

# changing size of circles in a rad grid depending the mouseposition
#
# MOUSE
# position x/y        : module size and offset z
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *

tile_count = 20
module_color = color(0.0)
module_alpha = 180
act_random_seed = 0
max_distance = 500


def setup():
    size(600, 600)
    mode_3d()


def draw():
    background(1.0)
    no_fill()

    random_seed(act_random_seed)

    stroke(module_color.with_alpha(module_alpha / 255))
    stroke_weight(3)

    for grid_y in range(0, width, 25):
        for grid_x in range(0, height, 25):
            diameter = dist(mouse_x, mouse_y, grid_x, grid_y)
            diameter = diameter / max_distance * 40
            push_matrix()
            translate(grid_x - width / 2, grid_y - height / 2, diameter * 5)
            rect(0, 0, diameter, diameter)  # also nice: ellipse(...)
            pop_matrix()


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
