# P_2_1_1_01
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

# changing strokeweight and strokecaps on diagonals in a grid
#
# MOUSE
# position x          : left diagonal strokeweight
# position y          : right diagonal strokeweight
# left click          : new random layout
#
# KEYS
# 1                   : round strokecap
# 2                   : square strokecap
# 3                   : project strokecap
# s                   : save png
from datetime import datetime

from processing_py import *

tile_count = 20
act_random_seed = 0

act_stroke_cap = ROUND


def setup():
    size(600, 600)


def draw():
    background(1.0)
    no_fill()
    stroke_cap(act_stroke_cap)

    random_seed(act_random_seed)

    for grid_y in range(tile_count):
        for grid_x in range(tile_count):
            pos_x = width // tile_count * grid_x
            pos_y = height // tile_count * grid_y

            toggle = int(random(0, 2))

            if toggle == 0:
                stroke_weight(mouse_x / 20)
                line(pos_x, pos_y, pos_x + width / tile_count, pos_y + height / tile_count)
            if toggle == 1:
                stroke_weight(mouse_y / 20)
                line(pos_x, pos_y + width / tile_count, pos_x + height / tile_count, pos_y)


def mouse_pressed():
    global act_random_seed
    act_random_seed = int(random(100000))


def key_released():
    global act_stroke_cap
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        act_stroke_cap = ROUND
    if key == "2":
        act_stroke_cap = SQUARE
    if key == "3":
        act_stroke_cap = PROJECT


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
