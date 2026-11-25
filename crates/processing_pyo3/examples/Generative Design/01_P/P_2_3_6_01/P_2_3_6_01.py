# P_2_3_6_01
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

# draw tool. draws a specific module according to
# its east, south, west and north neighbours.
#
# MOUSE
# drag left           : draw new module
# drag right          : delete a module
#
# KEYS
# del, backspace      : clear screen
# g                   : toogle. show grid
# d                   : toogle. show module values
# s                   : save png
from datetime import datetime

from processing_py import *

font = None
modules = []

tile_size = 30
grid_resolution_x = 0
grid_resolution_y = 0
tiles = []

draw_grid_on = True
debug_mode = False


def setup():
    global font, grid_resolution_x, grid_resolution_y
    # use full screen size
    size(display_width, display_height)  # GAP: display_width / display_height are 0 until the first draw(), so the window cannot be created
    # size(720, 540)
    cursor(CROSS)
    font = create_font("sans-serif")  # GAP: create_font() raises "Font not found" for the generic family "sans-serif"
    text_font(font)
    text_size(8)
    text_align(CENTER, CENTER)

    grid_resolution_x = round(width / tile_size) + 2
    grid_resolution_y = round(height / tile_size) + 2
    for grid_x in range(grid_resolution_x):
        tiles.append(["0"] * grid_resolution_y)
    init_tiles()

    # load svg modules
    for i in range(16):
        modules.append(load_shape("data/" + str(i).zfill(2) + ".svg"))  # GAP: load_shape() (SVG) is missing


def draw():
    background(1.0)

    if mouse_is_pressed and mouse_button == LEFT:
        set_tile()
    if mouse_is_pressed and mouse_button == RIGHT:
        unset_tile()

    if draw_grid_on:
        draw_grid()
    draw_modules()


def init_tiles():
    for grid_y in range(grid_resolution_y):
        for grid_x in range(grid_resolution_x):
            tiles[grid_x][grid_y] = "0"


def set_tile():
    # convert mouse position to grid coordinates
    grid_x = floor(mouse_x / tile_size) + 1
    grid_x = constrain(grid_x, 1, grid_resolution_x - 2)
    grid_y = floor(mouse_y / tile_size) + 1
    grid_y = constrain(grid_y, 1, grid_resolution_y - 2)
    tiles[grid_x][grid_y] = "1"


def unset_tile():
    grid_x = floor(mouse_x / tile_size) + 1
    grid_x = constrain(grid_x, 1, grid_resolution_x - 2)
    grid_y = floor(mouse_y / tile_size) + 1
    grid_y = constrain(grid_y, 1, grid_resolution_y - 2)
    tiles[grid_x][grid_y] = "0"


def draw_grid():
    rect_mode(CENTER)
    for grid_y in range(grid_resolution_y):
        for grid_x in range(grid_resolution_x):
            pos_x = tile_size * grid_x - tile_size / 2
            pos_y = tile_size * grid_y - tile_size / 2
            stroke_weight(0.15)
            fill(1.0)
            if debug_mode:
                if tiles[grid_x][grid_y] == "1":
                    fill(220 / 255)
            rect(pos_x, pos_y, tile_size, tile_size)


def draw_modules():
    shape_mode(CENTER)  # GAP: shape_mode()
    for grid_y in range(1, grid_resolution_y - 1):
        for grid_x in range(1, grid_resolution_x - 1):
            # use only active tiles
            if tiles[grid_x][grid_y] == "1":
                # check the four neighbours, each can be 0 or 1
                east = str(tiles[grid_x + 1][grid_y])
                south = str(tiles[grid_x][grid_y + 1])
                west = str(tiles[grid_x - 1][grid_y])
                north = str(tiles[grid_x][grid_y - 1])
                # create a binary result out of it, eg. 1011
                binary_result = north + west + south + east
                # convert the binary string to a decimal value from 0-15
                decimal_result = int(binary_result, 2)
                pos_x = tile_size * grid_x - tile_size / 2
                pos_y = tile_size * grid_y - tile_size / 2
                # decimal_result is the also the index for the shape array
                shape(modules[decimal_result], pos_x, pos_y, tile_size, tile_size)  # GAP: shape(s, x, y, w, h)

                if debug_mode:
                    fill(150 / 255)
                    text(str(decimal_result) + "\n" + binary_result, pos_x, pos_y)


def key_released():
    global draw_grid_on, debug_mode
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key_code == DELETE or key_code == BACKSPACE:
        init_tiles()
    if key == "g" or key == "G":
        draw_grid_on = not draw_grid_on
    if key == "d" or key == "D":
        debug_mode = not debug_mode


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
