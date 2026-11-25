# P_2_1_1_04
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

# shapes in a grid, that are always facing the mouse
#
# MOUSE
# position x/y        : position to face
#
# KEYS
# 1-7                 : choose shapes
# arrow up/down       : scale of shapes
# arrow left/right    : additional rotation of shapes
# c                   : toggle. color mode
# d                   : toggle. size depending on distance
# g                   : toggle. grid resolution
# s                   : save png
from datetime import datetime

from processing_py import *

current_shape = None

tile_count = 10
tile_width = 0
tile_height = 0
shape_size = 50
new_shape_size = shape_size
shape_angle = 0
max_dist = 0

shape_color = color(0.0, 130 / 255, 164 / 255)
fill_mode = 0
size_mode = 0


def setup():
    global tile_width, tile_height, max_dist, current_shape
    size(600, 600)
    background(1.0)

    tile_width = width / float(tile_count)
    tile_height = height / float(tile_count)
    max_dist = sqrt(sq(width) + sq(height))

    current_shape = load_shape("data/module_1.svg")  # GAP: load_shape() (SVG) is missing


def draw():
    global new_shape_size
    background(1.0)

    for grid_y in range(tile_count):
        for grid_x in range(tile_count):
            pos_x = tile_width * grid_x + tile_width / 2
            pos_y = tile_height * grid_y + tile_width / 2

            # calculate angle between mouse position and actual position of the shape
            angle = atan2(mouse_y - pos_y, mouse_x - pos_x) + radians(shape_angle)

            if size_mode == 0:
                new_shape_size = shape_size
            if size_mode == 1:
                new_shape_size = shape_size * 1.5 - remap(dist(mouse_x, mouse_y, pos_x, pos_y), 0, 500, 5, shape_size)
            if size_mode == 2:
                new_shape_size = remap(dist(mouse_x, mouse_y, pos_x, pos_y), 0, 500, 5, shape_size)

            if fill_mode == 0:
                current_shape.enable_style()  # GAP: Shape.enable_style()
            if fill_mode == 1:
                current_shape.disable_style()  # GAP: Shape.disable_style(); ignore the colors in the SVG
                fill(shape_color)
            if fill_mode == 2:
                current_shape.disable_style()  # GAP: Shape.disable_style(); ignore the colors in the SVG
                a = remap(dist(mouse_x, mouse_y, pos_x, pos_y), 0, max_dist, 1.0, 0.0)
                fill(shape_color.with_alpha(a))
            if fill_mode == 3:
                current_shape.disable_style()  # GAP: Shape.disable_style(); ignore the colors in the SVG
                a = remap(dist(mouse_x, mouse_y, pos_x, pos_y), 0, max_dist, 0.0, 1.0)
                fill(shape_color.with_alpha(a))

            push_matrix()
            translate(pos_x, pos_y)
            rotate(angle)
            shape_mode(CENTER)  # GAP: shape_mode() is missing

            no_stroke()
            shape(current_shape, 0, 0, new_shape_size, new_shape_size)  # GAP: shape(s, x, y, w, h)
            pop_matrix()


def key_pressed():
    global fill_mode, size_mode, tile_count, tile_width, tile_height
    global current_shape, shape_size, shape_angle
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == "c" or key == "C":
        fill_mode = (fill_mode + 1) % 4
    if key == "d" or key == "D":
        size_mode = (size_mode + 1) % 3

    if key == "g" or key == "G":
        tile_count = tile_count + 5
        if tile_count > 20:
            tile_count = 10
        tile_width = width / float(tile_count)
        tile_height = height / float(tile_count)

    if key == "1":
        current_shape = load_shape("data/module_1.svg")  # GAP: load_shape() (SVG) is missing
    if key == "2":
        current_shape = load_shape("data/module_2.svg")  # GAP: load_shape() (SVG) is missing
    if key == "3":
        current_shape = load_shape("data/module_3.svg")  # GAP: load_shape() (SVG) is missing
    if key == "4":
        current_shape = load_shape("data/module_4.svg")  # GAP: load_shape() (SVG) is missing
    if key == "5":
        current_shape = load_shape("data/module_5.svg")  # GAP: load_shape() (SVG) is missing
    if key == "6":
        current_shape = load_shape("data/module_6.svg")  # GAP: load_shape() (SVG) is missing
    if key == "7":
        current_shape = load_shape("data/module_7.svg")  # GAP: load_shape() (SVG) is missing

    if key_code == UP:
        shape_size += 5
    if key_code == DOWN:
        shape_size = max(shape_size - 5, 5)
    if key_code == LEFT_ARROW:
        shape_angle -= 5
    if key_code == RIGHT_ARROW:
        shape_angle += 5


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
