# P_2_1_3_01
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

# changing circle amount, size and position in a grid
#
# MOUSE
# position x          : circle amount and size
# position y          : circle position
# left click          : random position
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *

tile_count_x = 10
tile_count_y = 10
tile_width = 0
tile_height = 0

count = 0
color_step = 15
circle_count = 0
end_size = 0
end_offset = 0

act_random_seed = 0


def setup():
    global tile_width, tile_height
    size(800, 800)
    tile_width = width / tile_count_x
    tile_height = height / tile_count_y


def draw():
    global circle_count, end_size, end_offset
    no_fill()
    stroke(0.0, 128 / 255)
    background(1.0)
    random_seed(act_random_seed)

    translate((width / tile_count_x) / 2, (height / tile_count_y) / 2)

    circle_count = int(mouse_x) // 30 + 1
    end_size = remap(mouse_x, 0, width, tile_width / 2.0, 0)
    end_offset = remap(mouse_y, 0, height, 0, (tile_width - end_size) / 2)

    for grid_y in range(tile_count_y + 1):
        for grid_x in range(tile_count_x + 1):
            push_matrix()
            translate(tile_width * grid_x, tile_height * grid_y)
            scale(1, tile_height / tile_width)

            toggle = int(random(0, 4))
            if toggle == 0:
                rotate(-HALF_PI)
            if toggle == 1:
                rotate(0)
            if toggle == 2:
                rotate(HALF_PI)
            if toggle == 3:
                rotate(PI)

            # draw module
            for i in range(circle_count):
                diameter = remap(i, 0, circle_count - 1, tile_width, end_size)
                offset = remap(i, 0, circle_count - 1, 0, end_offset)
                ellipse(offset, 0, diameter, diameter)
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
