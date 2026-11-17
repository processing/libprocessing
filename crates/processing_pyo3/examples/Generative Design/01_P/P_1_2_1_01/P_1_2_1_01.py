# P_1_2_1_01
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

# shows how to interpolate colors in different styles/ color modes
#
# MOUSE
# left click          : new random color set
# position x          : interpolation resolution
# position y          : row count
#
# KEYS
# 1-2                 : switch interpolation style
# s                   : save png
# c                   : save color palette
import struct
from datetime import datetime

from processing_py import *

tile_count_x = 2
tile_count_y = 10

colors_left = [None] * tile_count_y
colors_right = [None] * tile_count_y
colors = []

interpolate_shortest = True


def setup():
    size(800, 800)
    no_stroke()
    shake_colors()


def draw():
    global tile_count_x, tile_count_y, colors
    tile_count_x = int(remap(mouse_x, 0, width, 2, 100))
    tile_count_y = int(remap(mouse_y, 0, height, 2, 10))
    tile_width = width / tile_count_x
    tile_height = height / tile_count_y

    # just for ase export
    colors = [None] * (tile_count_x * tile_count_y)
    i = 0

    for grid_y in range(tile_count_y):
        col1 = colors_left[grid_y]
        col2 = colors_right[grid_y]

        for grid_x in range(tile_count_x):
            amount = remap(grid_x, 0, tile_count_x - 1, 0, 1)

            if interpolate_shortest:
                # interpolate in rgb
                inter_col = lerp_color(col1, col2, amount)
            else:
                # interpolate in hsb
                inter_hue = lerp(hue(col1), hue(col2), amount)
                inter_saturation = lerp(saturation(col1), saturation(col2), amount)
                inter_brightness = lerp(brightness(col1), brightness(col2), amount)
                inter_col = hsva(inter_hue, inter_saturation, inter_brightness)
            fill(inter_col)

            pos_x = tile_width * grid_x
            pos_y = tile_height * grid_y
            rect(pos_x, pos_y, tile_width, tile_height)

            # just for ase export
            colors[i] = inter_col
            i += 1


def shake_colors():
    for i in range(len(colors_left)):
        colors_left[i] = hsva(random(0, 60), random(0, 100) / 100, 1.0)
        colors_right[i] = hsva(random(160, 190), 1.0, random(0, 100) / 100)


def mouse_released():
    shake_colors()


def key_released():
    global interpolate_shortest
    if key == "c" or key == "C":
        save_ase(colors, timestamp() + ".ase")
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        interpolate_shortest = True
    if key == "2":
        interpolate_shortest = False


def save_ase(colors, filename):
    # write an ase file (adobe swatch exchange)
    f = open(filename, "wb")
    f.write(b"ASEF")
    f.write(struct.pack(">HHI", 1, 0, len(colors)))
    for c in colors:
        name = c.to_hex()
        block = struct.pack(">H", len(name) + 1) + name.encode("utf-16-be") + b"\x00\x00"
        block = block + b"RGB " + struct.pack(">fff", red(c), green(c), blue(c))
        block = block + struct.pack(">H", 0)
        f.write(struct.pack(">HI", 1, len(block)))
        f.write(block)
    f.close()

# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
