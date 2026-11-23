# P_2_1_1_03
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

# changing number, color and strokeweight on diagonals in a grid
#
# MOUSE
# position x          : diagonal strokeweight
# position y          : number diagonals
# left click          : new random layout
#
# KEYS
# s                   : save png
# 1                   : color left diagonal
# 2                   : color right diagonal
# 3                   : switch transparency left diagonal on/off
# 4                   : switch transparency right diagonal on/off
# 0                   : default
from datetime import datetime

from processing_py import *

color_back = color(1.0)
color_left = color(0.0)
color_right = color(0.0)

tile_count = 1
transparent_left = False
transparent_right = False
alpha_left = 100
alpha_right = 100

act_random_seed = 0


def setup():
    global color_left
    size(600, 600)

    color_left = hsva(323, 1.0, 0.77)


def draw():
    global tile_count, alpha_left, alpha_right
    background(color_back)
    no_fill()
    random_seed(act_random_seed)
    stroke_weight(mouse_x // 15)

    tile_count = mouse_y // 15

    for grid_y in range(int(tile_count)):
        for grid_x in range(int(tile_count)):
            pos_x = width / tile_count * grid_x
            pos_y = height / tile_count * grid_y

            if transparent_left == True:
                alpha_left = grid_y * 10
            else:
                alpha_left = 100

            if transparent_right == True:
                alpha_right = 100 - grid_y * 10
            else:
                alpha_right = 100

            toggle = int(random(0, 2))

            if toggle == 0:
                stroke(color_left.with_alpha(alpha_left / 100))
                line(pos_x, pos_y, pos_x + (width / tile_count) / 2, pos_y + height / tile_count)
                line(pos_x + (width / tile_count) / 2, pos_y, pos_x + (width / tile_count), pos_y + height / tile_count)
            if toggle == 1:
                stroke(color_right.with_alpha(alpha_right / 100))
                line(pos_x, pos_y + width / tile_count, pos_x + (height / tile_count) / 2, pos_y)
                line(pos_x + (height / tile_count) / 2, pos_y + width / tile_count, pos_x + (height / tile_count), pos_y)


def mouse_pressed():
    global act_random_seed
    act_random_seed = int(random(100000))


def key_released():
    global color_left, color_right, transparent_left, transparent_right
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        if color_left == hsva(273, 0.73, 0.51):
            color_left = hsva(323, 1.0, 0.77)
        else:
            color_left = hsva(273, 0.73, 0.51)
            #      color_left = color(0.0)
    if key == "2":
        if color_right == color(0.0):
            color_right = hsva(192, 1.0, 0.64)
        else:
            color_right = color(0.0)
    if key == "3":
        transparent_left = not transparent_left
    if key == "4":
        transparent_right = not transparent_right

    if key == "0":
        transparent_left = False
        transparent_right = False
        color_left = hsva(323, 1.0, 0.77)
        color_right = color(0.0)


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
