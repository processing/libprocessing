# P_2_1_3_02
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

# draw a module made of lines in a grid
#
# MOUSE
# position x          : number of tiles horizontally
# position y          : number of tiles vertically
#
# KEYS
# 1-3                 : draw mode
# s                   : save png
from datetime import datetime

from processing_py import *

tile_count_x = 5
tile_count_y = 5

count = 10
color_step = 20

line_weight = 0
stroke_color = 0

background_color = 0.0

draw_mode = 1


def setup():
    size(600, 600)


def draw():
    global tile_count_x, tile_count_y, line_weight, stroke_color, background_color
    stroke_weight(0.5)
    stroke_cap(ROUND)

    tile_count_x = int(mouse_x) // 30 + 1
    tile_count_y = int(mouse_y) // 30 + 1

    background(background_color)

    for grid_y in range(tile_count_y + 1):
        for grid_x in range(tile_count_x + 1):
            tile_width = width / tile_count_x
            tile_height = height / tile_count_y
            pos_x = tile_width * grid_x
            pos_y = tile_height * grid_y

            x1 = tile_width / 2
            y1 = tile_height / 2
            x2 = 0
            y2 = 0

            push_matrix()
            translate(pos_x, pos_y)

            for side in range(4):
                for i in range(count):
                    # move end point around the four sides of the tile
                    if side == 0:
                        x2 += tile_width / count
                        y2 = 0
                    if side == 1:
                        x2 = tile_width
                        y2 += tile_height / count
                    if side == 2:
                        x2 -= tile_width / count
                        y2 = tile_height
                    if side == 3:
                        x2 = 0
                        y2 -= tile_height / count

                    # adjust weight and color of the line
                    if i < count // 2:
                        line_weight += 1
                        stroke_color += 60
                    else:
                        line_weight -= 1
                        stroke_color -= 60

                    # set colors depending on draw mode
                    if draw_mode == 1:
                        background_color = 1.0
                        stroke(0.0)
                    elif draw_mode == 2:
                        background_color = 1.0
                        stroke(0.0)
                        stroke_weight(line_weight)  # GAP: a zero-weight line drawn last in the frame makes every earlier stroke vanish (line_weight ends each tile at 0)
                    elif draw_mode == 3:
                        background_color = 0.0
                        stroke(stroke_color / 360)
                        stroke_weight(mouse_x // 100)

                    # draw the line
                    line(x1, y1, x2, y2)

            pop_matrix()


def key_pressed():
    global draw_mode
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        draw_mode = 1
    if key == "2":
        draw_mode = 2
    if key == "3":
        draw_mode = 3


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
