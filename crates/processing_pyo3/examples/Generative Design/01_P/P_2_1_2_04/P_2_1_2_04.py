# P_2_1_2_04
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

# moving corners of rectangles in a grid
#
# MOUSE
# position x          : corner position offset x
# position y          : corner position offset y
# left click          : random position
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *

tile_count = 20
rect_size = 30

act_random_seed = 0


def setup():
    size(600, 600)


def draw():
    background(1.0)
    no_stroke()

    fill(hsva(192, 1.0, 0.64, 0.6))

    random_seed(act_random_seed)

    for grid_y in range(tile_count):
        for grid_x in range(tile_count):
            pos_x = width // tile_count * grid_x
            pos_y = height // tile_count * grid_y

            shift_x1 = mouse_x // 20 * random(-1, 1)
            shift_y1 = mouse_y // 20 * random(-1, 1)
            shift_x2 = mouse_x // 20 * random(-1, 1)
            shift_y2 = mouse_y // 20 * random(-1, 1)
            shift_x3 = mouse_x // 20 * random(-1, 1)
            shift_y3 = mouse_y // 20 * random(-1, 1)
            shift_x4 = mouse_x // 20 * random(-1, 1)
            shift_y4 = mouse_y // 20 * random(-1, 1)

            begin_shape()
            vertex(pos_x + shift_x1, pos_y + shift_y1)
            vertex(pos_x + rect_size + shift_x2, pos_y + shift_y2)
            vertex(pos_x + rect_size + shift_x3, pos_y + rect_size + shift_y3)
            vertex(pos_x + shift_x4, pos_y + rect_size + shift_y4)
            end_shape(CLOSE)


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
