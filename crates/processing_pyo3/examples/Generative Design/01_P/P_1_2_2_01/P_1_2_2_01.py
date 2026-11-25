# P_1_2_2_01
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

# extract and sort the color palette of an image
#
# MOUSE
# position x          : resolution
#
# KEYS
# 1-3                 : load different images
# 4                   : no color sorting
# 5                   : sort colors on hue
# 6                   : sort colors on saturation
# 7                   : sort colors on brightness
# 8                   : sort colors on grayscale (luminance)
# s                   : save png
# c                   : save color palette
import struct
from datetime import datetime

from processing_py import *

# sort modes of the generativedesign library
HUE = "hue"
SATURATION = "saturation"
BRIGHTNESS = "brightness"
GRAYSCALE = "grayscale"

img = None
colors = []

sort_mode = None


def setup():
    global img
    size(600, 600)
    no_stroke()
    no_cursor()
    img = load_image("data/pic1.jpg")


def draw():
    global colors
    tile_count = int(width / max(mouse_x, 5))
    rect_size = width / tile_count

    # get colors from image
    i = 0
    colors = [None] * (tile_count * tile_count)
    for grid_y in range(tile_count):
        for grid_x in range(tile_count):
            px = int(grid_x * rect_size)
            py = int(grid_y * rect_size)
            colors[i] = img.get(px, py)
            i += 1

    # sort colors
    if sort_mode != None:
        colors = sort_colors(colors, sort_mode)

    # draw grid
    i = 0
    for grid_y in range(tile_count):
        for grid_x in range(tile_count):
            fill(colors[i])
            rect(grid_x * rect_size, grid_y * rect_size, rect_size, rect_size)
            i += 1


def key_released():
    global img, sort_mode
    if key == "c" or key == "C":
        save_ase(colors, timestamp() + ".ase")
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        img = load_image("data/pic1.jpg")
    if key == "2":
        img = load_image("data/pic2.jpg")
    if key == "3":
        img = load_image("data/pic3.jpg")

    if key == "4":
        sort_mode = None
    if key == "5":
        sort_mode = HUE
    if key == "6":
        sort_mode = SATURATION
    if key == "7":
        sort_mode = BRIGHTNESS
    if key == "8":
        sort_mode = GRAYSCALE


# luminance of a color
def grayscale(c):
    return red(c) * 0.222 + green(c) * 0.707 + blue(c) * 0.071


# sorts a list of colors according to the given method
def sort_colors(colors, method):
    # sort hue
    if method == HUE:
        colors.sort(key=hue)
    # sort saturation
    if method == SATURATION:
        colors.sort(key=saturation)
    # sort brightness
    if method == BRIGHTNESS:
        colors.sort(key=brightness)
    # sort grayscale
    if method == GRAYSCALE:
        colors.sort(key=grayscale)
    return colors


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
