# P_2_1_3_03
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
# position x          : module detail
# position y          : module parameter
#
# KEYS
# 1-3                 : draw mode
# arrow left/right    : number of tiles horizontally
# arrow up/down       : number of tiles vertically
# s                   : save png
from datetime import datetime

from processing_py import *

tile_count_x = 6
tile_count_y = 6
count = 0

draw_mode = 1


def setup():
    size(600, 600)


def draw():
    global count
    rect_mode(CENTER)
    # no_stroke()
    stroke(0.0)
    no_fill()
    background(1.0)

    count = int(mouse_x) // 20 + 5
    para = mouse_y / height - 0.5

    for grid_y in range(tile_count_y + 1):
        for grid_x in range(tile_count_x + 1):
            tile_width = width / tile_count_x
            tile_height = height / tile_count_y
            pos_x = tile_width * grid_x + tile_width / 2
            pos_y = tile_height * grid_y + tile_height / 2

            push_matrix()
            translate(pos_x, pos_y)

            # switch between modules
            if draw_mode == 1:
                translate(-tile_width / 2, -tile_height / 2)
                for i in range(count):
                    line(0, (para + 0.5) * tile_height, tile_width, i * tile_height / count)
                    line(0, i * tile_height / count, tile_width, tile_height - (para + 0.5) * tile_height)
            elif draw_mode == 2:
                for i in range(count + 1):
                    line(para * tile_width, para * tile_height, tile_width / 2, (i / count - 0.5) * tile_height)
                    line(para * tile_width, para * tile_height, -tile_width / 2, (i / count - 0.5) * tile_height)
                    line(para * tile_width, para * tile_height, (i / count - 0.5) * tile_width, tile_height / 2)
                    line(para * tile_width, para * tile_height, (i / count - 0.5) * tile_width, -tile_height / 2)
            elif draw_mode == 3:
                for i in range(count + 1):
                    line(0, para * tile_height, tile_width / 2, (i / count - 0.5) * tile_height)
                    line(0, para * tile_height, -tile_width / 2, (i / count - 0.5) * tile_height)
                    line(0, para * tile_height, (i / count - 0.5) * tile_width, tile_height / 2)
                    line(0, para * tile_height, (i / count - 0.5) * tile_width, -tile_height / 2)

            pop_matrix()


def key_released():
    global draw_mode, tile_count_x, tile_count_y
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        draw_mode = 1
    if key == "2":
        draw_mode = 2
    if key == "3":
        draw_mode = 3

    if key_code == DOWN:
        tile_count_y = max(tile_count_y - 1, 1)
    if key_code == UP:
        tile_count_y += 1
    if key_code == LEFT_ARROW:
        tile_count_x = max(tile_count_x - 1, 1)
    if key_code == RIGHT_ARROW:
        tile_count_x += 1


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
