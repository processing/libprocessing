# P_1_2_3_01
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

# generates specific color palettes
#
# MOUSE
# position x/y        : row and coloum count
#
# KEYS
# 0-9                 : creates specific color palettes
# s                   : save png
# c                   : save color palette
import struct
from datetime import datetime

from processing_py import *

tile_count_x = 50
tile_count_y = 10

# arrays for color components values
hue_values = [0] * tile_count_x
saturation_values = [0] * tile_count_x
brightness_values = [0] * tile_count_x


def setup():
    size(800, 800)
    no_stroke()

    # init with random values
    for i in range(tile_count_x):
        hue_values[i] = int(random(0, 360))
        saturation_values[i] = int(random(0, 100))
        brightness_values[i] = int(random(0, 100))


def draw():
    # white back
    background(1.0)

    # count every tile
    counter = 0

    # map mouse to grid resolution
    current_tile_count_x = int(remap(mouse_x, 0, width, 1, tile_count_x))
    current_tile_count_y = int(remap(mouse_y, 0, height, 1, tile_count_y))
    tile_width = width / current_tile_count_x
    tile_height = height / current_tile_count_y

    for grid_y in range(tile_count_y):
        for grid_x in range(tile_count_x):
            pos_x = tile_width * grid_x
            pos_y = tile_height * grid_y
            index = counter % current_tile_count_x

            # get component color values
            fill(hsva(hue_values[index], saturation_values[index] / 100, brightness_values[index] / 100))
            rect(pos_x, pos_y, tile_width, tile_height)
            counter += 1


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == "c" or key == "C":
        # ------ save an ase file (adobe swatch export) ------
        # create palette
        colors = [None] * len(hue_values)
        for i in range(len(hue_values)):
            colors[i] = hsva(hue_values[i], saturation_values[i] / 100, brightness_values[i] / 100)
        save_ase(colors, timestamp() + ".ase")

    if key == "1":
        for i in range(tile_count_x):
            hue_values[i] = int(random(0, 360))
            saturation_values[i] = int(random(0, 100))
            brightness_values[i] = int(random(0, 100))
    if key == "2":
        for i in range(tile_count_x):
            hue_values[i] = int(random(0, 360))
            saturation_values[i] = int(random(0, 100))
            brightness_values[i] = 100
    if key == "3":
        for i in range(tile_count_x):
            hue_values[i] = int(random(0, 360))
            saturation_values[i] = 100
            brightness_values[i] = int(random(0, 100))

    if key == "4":
        for i in range(tile_count_x):
            hue_values[i] = 0
            saturation_values[i] = 0
            brightness_values[i] = int(random(0, 100))
    if key == "5":
        for i in range(tile_count_x):
            hue_values[i] = 195
            saturation_values[i] = 100
            brightness_values[i] = int(random(0, 100))
    if key == "6":
        for i in range(tile_count_x):
            hue_values[i] = 195
            saturation_values[i] = int(random(0, 100))
            brightness_values[i] = 100

    if key == "7":
        for i in range(tile_count_x):
            hue_values[i] = int(random(0, 180))
            saturation_values[i] = int(random(80, 100))
            brightness_values[i] = int(random(50, 90))
    if key == "8":
        for i in range(tile_count_x):
            hue_values[i] = int(random(180, 360))
            saturation_values[i] = int(random(80, 100))
            brightness_values[i] = int(random(50, 90))

    if key == "9":
        for i in range(tile_count_x):
            if i % 2 == 0:
                hue_values[i] = int(random(0, 360))
                saturation_values[i] = 100
                brightness_values[i] = int(random(0, 100))
            else:
                hue_values[i] = 195
                saturation_values[i] = int(random(0, 100))
                brightness_values[i] = 100
    if key == "0":
        for i in range(tile_count_x):
            if i % 2 == 0:
                hue_values[i] = 192
                saturation_values[i] = int(random(0, 100))
                brightness_values[i] = int(random(10, 100))
            else:
                hue_values[i] = 273
                saturation_values[i] = int(random(0, 100))
                brightness_values[i] = int(random(10, 90))


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
