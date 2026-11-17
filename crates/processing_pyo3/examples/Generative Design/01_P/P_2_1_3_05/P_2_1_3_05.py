# P_2_1_3_05
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

# changing positions of stapled circles in a grid
#
# MOUSE
# position x          : horizontal position shift
# position y          : vertical position shift
# left click          : random position shift
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *

tile_count_x = 10
tile_count_y = 10
count = 0
color_step = 6
end_size = 0
step_size = 30
act_random_seed = 0


def setup():
    size(600, 600)


def draw():
    global step_size, end_size
    no_stroke()
    background(1.0)
    random_seed(act_random_seed)
    step_size = int(mouse_x) // 10
    end_size = int(mouse_y) // 10
    for grid_y in range(tile_count_y + 1):
        for grid_x in range(tile_count_x + 1):
            # kachelgr?ssen und positionen
            tile_width = width / tile_count_x
            tile_height = height / tile_count_y
            pos_x = tile_width * grid_x
            pos_y = tile_height * grid_y
            toggle = int(random(4))
            if toggle == 0:
                # modul
                for i in range(step_size):
                    diameter = remap(i, 0, step_size, tile_width, end_size)
                    fill((360 - (i * color_step)) / 360)
                    ellipse(pos_x + i, pos_y, diameter, diameter)
            elif toggle == 1:
                # modul
                for i in range(step_size):
                    diameter = remap(i, 0, step_size, tile_height, end_size)
                    fill((360 - (i * color_step)) / 360)
                    ellipse(pos_x, pos_y + i, diameter, diameter)
            elif toggle == 2:
                # modul
                for i in range(step_size):
                    diameter = remap(i, 0, step_size, tile_width, end_size)
                    fill((360 - (i * color_step)) / 360)
                    ellipse(pos_x - i, pos_y, diameter, diameter)
            elif toggle == 3:
                # modul
                for i in range(step_size):
                    diameter = remap(i, 0, step_size, tile_height, end_size)
                    fill((360 - (i * color_step)) / 360)
                    ellipse(pos_x, pos_y - i, diameter, diameter)


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
