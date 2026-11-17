# P_2_1_2_02
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

# changing module color and positions in a grid
#
# MOUSE
# position x          : offset x
# position y          : offset y
# left click          : random position
#
# KEYS
# 1-3                 : different sets of colors
# 0                   : default
# arrow up/down       : background module size
# arrow left/right    : foreground module size
# s                   : save png
from datetime import datetime

from processing_py import *

module_color_background = color(0.0)
module_color_foreground = color(1.0)

module_alpha_background = 100
module_alpha_foreground = 100

module_radius_background = 30
module_radius_foreground = 15

back_color = color(1.0)


tile_count = 20
act_random_seed = 0


def setup():
    size(600, 600)


def draw():
    translate(width / tile_count / 2, height / tile_count / 2)

    background(back_color)
    no_stroke()

    random_seed(act_random_seed)

    for grid_y in range(tile_count):
        for grid_x in range(tile_count):
            pos_x = width / tile_count * grid_x
            pos_y = height / tile_count * grid_y

            shift_x = random(-1, 1) * mouse_x / 20
            shift_y = random(-1, 1) * mouse_y / 20

            fill(module_color_background.with_alpha(module_alpha_background / 100))
            ellipse(pos_x + shift_x, pos_y + shift_y, module_radius_background, module_radius_background)

    for grid_y in range(tile_count):
        for grid_x in range(tile_count):
            pos_x = width / tile_count * grid_x
            pos_y = height / tile_count * grid_y

            fill(module_color_foreground.with_alpha(module_alpha_foreground / 100))
            ellipse(pos_x, pos_y, module_radius_foreground, module_radius_foreground)


def mouse_pressed():
    global act_random_seed
    act_random_seed = int(random(100000))


def key_released():
    global module_color_background, module_color_foreground
    global module_alpha_background, module_alpha_foreground
    global module_radius_background, module_radius_foreground
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        if module_color_background == color(0.0):
            module_color_background = hsva(273, 0.73, 0.51)
        else:
            module_color_background = color(0.0)
    if key == "2":
        if module_color_foreground == color(1.0):
            module_color_foreground = hsva(323, 1.0, 0.77)
        else:
            module_color_foreground = color(1.0)

    if key == "3":
        if module_alpha_background == 100:
            module_alpha_background = 50
            module_alpha_foreground = 50
        else:
            module_alpha_background = 100
            module_alpha_foreground = 100

    if key == "0":
        module_color_background = color(0.0)
        module_color_foreground = color(1.0)
        module_alpha_background = 100
        module_alpha_foreground = 100
        module_radius_background = 20
        module_radius_foreground = 10

    if key_code == UP:
        module_radius_background += 2
    if key_code == DOWN:
        module_radius_background = max(module_radius_background - 2, 10)
    if key_code == LEFT_ARROW:
        module_radius_foreground = max(module_radius_foreground - 2, 5)
    if key_code == RIGHT_ARROW:
        module_radius_foreground += 2


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
