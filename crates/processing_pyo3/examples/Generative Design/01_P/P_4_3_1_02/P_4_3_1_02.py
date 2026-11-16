# P_4_3_1_02
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

# pixel mapping. each pixel is translated into a new element (svg file).
# take care to sort the svg file according to their greyscale value.
# see also "_4_3_1_02_analyse_svg_grayscale.pde"
#
# KEYS
# s                   : save png
import os
from datetime import datetime

from processing_py import *

shapes = []
shape_count = 0

img = None


def setup():
    global img, shapes, shape_count
    size(600, 900)
    img = load_image("data/pic.png")
    print(str(img.width) + " x " + str(img.height))

    # ------ load shapes ------
    # replace this location with a folder on your machine
    dir = "data"

    if os.path.isdir(dir):
        contents = os.listdir(dir)
        svg_files = []
        for i in range(len(contents)):
            # skip hidden files and folders starting with a dot, load .svg files only
            if contents[i][0] == ".":
                continue
            elif contents[i].lower().endswith(".svg"):
                child_file = os.path.join(dir, contents[i])
                print(i, child_file)
                svg_files.append(child_file)

        svg_files = sorted(svg_files)

        shape_count = len(svg_files)
        shapes = [None] * shape_count
        for i in range(shape_count):
            shapes[i] = load_shape(svg_files[i])  # GAP: load_shape() (SVG) is missing


def draw():
    background(1.0)

    mouse_x_factor = remap(mouse_x, 0, width, 0.05, 1)
    mouse_y_factor = remap(mouse_y, 0, height, 0.05, 1)

    for grid_x in range(img.width):
        for grid_y in range(img.height):
            # grid position + tile size
            tile_width = width / img.width
            tile_height = height / img.height
            pos_x = tile_width * grid_x
            pos_y = tile_height * grid_y

            # get current color
            c = img.pixels[grid_y * img.width + grid_x]
            # greyscale conversion
            greyscale = round(red(c) * 255 * 0.222 + green(c) * 255 * 0.707 + blue(c) * 255 * 0.071)

            gradient_to_index = round(remap(greyscale, 0, 255, 0, shape_count - 1))
            shape(shapes[gradient_to_index], pos_x, pos_y, tile_width, tile_height)  # GAP: shape(s, x, y, w, h)


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
