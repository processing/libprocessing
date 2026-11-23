# P_2_3_6_02
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
# with switchable tileset
#
# MOUSE
# drag left           : draw new module
# drag right          : delete a module
#
# KEYS
# 1-8                 : switch tileset
# y,x,c,v,b           : switch colors
# del, backspace      : clear screen
# r                   : random tiles
# s                   : save png
# g                   : toogle. show grid
# d                   : toogle. show module values
from datetime import datetime

from processing_py import *

tile_size = 30
grid_resolution_x = 0
grid_resolution_y = 0
draw_grid_on = True
tiles = []
tile_colors = []
active_tile_color = color(0.0)

random_mode = False

modules_a = []
modules_b = []
modules_c = []
modules_d = []
modules_e = []
modules_f = []
modules_g = []
modules_h = []

modules = ["A", "B", "C", "D", "E", "F", "G", "H"]
active_moduls_set = "A"

font = None
debug_mode = False


def setup():
    global font, grid_resolution_x, grid_resolution_y
    # use full screen size
    size(display_width, display_height)  # GAP: display_width / display_height are 0 until the first draw(), so the window cannot be created
    # size(600, 600)
    cursor(CROSS)
    font = create_font("sans-serif")  # GAP: create_font() raises "Font not found" for the generic family "sans-serif"
    text_font(font)
    text_size(8)
    text_align(CENTER, CENTER)

    grid_resolution_x = round(width / tile_size) + 2
    grid_resolution_y = round(height / tile_size) + 2
    for grid_x in range(grid_resolution_x):
        tiles.append(["0"] * grid_resolution_y)
        tile_colors.append([None] * grid_resolution_y)
    init_tiles()

    # load svg modules
    for i in range(16):
        modules_a.append(load_shape("data/A_" + str(i).zfill(2) + ".svg"))  # GAP: load_shape() (SVG) is missing
        modules_b.append(load_shape("data/B_" + str(i).zfill(2) + ".svg"))  # GAP: load_shape() (SVG) is missing
        modules_c.append(load_shape("data/C_" + str(i).zfill(2) + ".svg"))  # GAP: load_shape() (SVG) is missing
        modules_d.append(load_shape("data/D_" + str(i).zfill(2) + ".svg"))  # GAP: load_shape() (SVG) is missing
        modules_e.append(load_shape("data/E_" + str(i).zfill(2) + ".svg"))  # GAP: load_shape() (SVG) is missing
        modules_f.append(load_shape("data/F_" + str(i).zfill(2) + ".svg"))  # GAP: load_shape() (SVG) is missing
        modules_g.append(load_shape("data/J_" + str(i).zfill(2) + ".svg"))  # GAP: load_shape() (SVG) is missing
        modules_h.append(load_shape("data/K_" + str(i).zfill(2) + ".svg"))  # GAP: load_shape() (SVG) is missing

        # disable Style
        modules_a[i].disable_style()  # GAP: Shape.disable_style()
        modules_b[i].disable_style()  # GAP: Shape.disable_style()
        modules_c[i].disable_style()  # GAP: Shape.disable_style()
        modules_d[i].disable_style()  # GAP: Shape.disable_style()
        modules_e[i].disable_style()  # GAP: Shape.disable_style()
        modules_f[i].disable_style()  # GAP: Shape.disable_style()
        modules_g[i].disable_style()  # GAP: Shape.disable_style()
        modules_h[i].disable_style()  # GAP: Shape.disable_style()


def draw():
    text_font(font)
    text_size(8)
    text_align(CENTER, CENTER)
    background(1.0)

    if mouse_is_pressed and mouse_button == LEFT:
        set_tile()
    if mouse_is_pressed and mouse_button == RIGHT:
        unset_tile()

    if draw_grid_on:
        draw_grid()
    draw_moduls()


def init_tiles():
    for grid_y in range(grid_resolution_y):
        for grid_x in range(grid_resolution_x):
            tiles[grid_x][grid_y] = "0"
            tile_colors[grid_x][grid_y] = hsva(random(255), 0.0, constrain(random(255) / 100, 0, 1))


def set_tile():
    # convert mouse position to grid coordinates
    grid_x = floor(mouse_x / tile_size) + 1
    grid_x = constrain(grid_x, 1, grid_resolution_x - 2)
    grid_y = floor(mouse_y / tile_size) + 1
    grid_y = constrain(grid_y, 1, grid_resolution_y - 2)
    tiles[grid_x][grid_y] = active_moduls_set
    tile_colors[grid_x][grid_y] = active_tile_color


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
                    fill(1.0)
            stroke(0.0)
            no_fill()
            rect(pos_x, pos_y, tile_size, tile_size)


def draw_moduls():
    global active_moduls_set
    if random_mode:
        active_moduls_set = modules[int(random(len(modules)))]

    shape_mode(CENTER)  # GAP: shape_mode()
    for grid_y in range(1, grid_resolution_y - 1):
        for grid_x in range(1, grid_resolution_x - 1):
            # use only active tiles
            current_tile = tiles[grid_x][grid_y]
            if current_tile != "0":
                binary_result = ""
                # check the four neighbours. is it active (not '0')?
                # create a binary result out of it, eg. 1011
                # binary_result = north + west + south + east
                # north
                if tiles[grid_x][grid_y - 1] != "0":
                    binary_result = "1"
                else:
                    binary_result = "0"
                # west
                if tiles[grid_x - 1][grid_y] != "0":
                    binary_result += "1"
                else:
                    binary_result += "0"
                # south
                if tiles[grid_x][grid_y + 1] != "0":
                    binary_result += "1"
                else:
                    binary_result += "0"
                # east
                if tiles[grid_x + 1][grid_y] != "0":
                    binary_result += "1"
                else:
                    binary_result += "0"

                # convert the binary string to a decimal value from 0-15
                decimal_result = int(binary_result, 2)
                pos_x = tile_size * grid_x - tile_size / 2
                pos_y = tile_size * grid_y - tile_size / 2

                fill(tile_colors[grid_x][grid_y])
                no_stroke()

                # decimal_result is the also the index for the shape array
                if current_tile == "A":
                    shape(modules_a[decimal_result], pos_x, pos_y, tile_size, tile_size)  # GAP: shape(s, x, y, w, h)
                elif current_tile == "B":
                    shape(modules_b[decimal_result], pos_x, pos_y, tile_size, tile_size)  # GAP: shape(s, x, y, w, h)
                elif current_tile == "C":
                    shape(modules_c[decimal_result], pos_x, pos_y, tile_size, tile_size)  # GAP: shape(s, x, y, w, h)
                elif current_tile == "D":
                    shape(modules_d[decimal_result], pos_x, pos_y, tile_size, tile_size)  # GAP: shape(s, x, y, w, h)
                elif current_tile == "E":
                    shape(modules_e[decimal_result], pos_x, pos_y, tile_size, tile_size)  # GAP: shape(s, x, y, w, h)
                elif current_tile == "F":
                    shape(modules_f[decimal_result], pos_x, pos_y, tile_size, tile_size)  # GAP: shape(s, x, y, w, h)
                elif current_tile == "G":
                    shape(modules_g[decimal_result], pos_x, pos_y, tile_size, tile_size)  # GAP: shape(s, x, y, w, h)
                elif current_tile == "H":
                    shape(modules_h[decimal_result], pos_x, pos_y, tile_size, tile_size)  # GAP: shape(s, x, y, w, h)

                if debug_mode:
                    fill(1.0)
                    text(current_tile + "\n" + str(decimal_result) + "\n" + binary_result, pos_x, pos_y)


def key_released():
    global draw_grid_on, debug_mode, random_mode, active_moduls_set, active_tile_color
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key_code == DELETE or key_code == BACKSPACE:
        init_tiles()
    if key == "g" or key == "G":
        draw_grid_on = not draw_grid_on
    if key == "d" or key == "D":
        debug_mode = not debug_mode
    if key == "r" or key == "R":
        random_mode = not random_mode
    if key == "1":
        active_moduls_set = "A"
    if key == "2":
        active_moduls_set = "B"
    if key == "3":
        active_moduls_set = "C"
    if key == "4":
        active_moduls_set = "D"
    if key == "5":
        active_moduls_set = "E"
    if key == "6":
        active_moduls_set = "F"
    if key == "7":
        active_moduls_set = "G"
    if key == "8":
        active_moduls_set = "H"

    if key == "y" or key == "Y":
        active_tile_color = color(0.0)
    if key == "x" or key == "X":
        active_tile_color = hsva(52, 1.0, 0.71)
    if key == "c" or key == "C":
        active_tile_color = hsva(192, 1.0, 0.64)
    if key == "v" or key == "V":
        active_tile_color = hsva(273, 0.73, 0.51)
    if key == "b" or key == "B":
        active_tile_color = hsva(323, 1.0, 0.77)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
