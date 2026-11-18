# P_4_3_1_02_analyse_svg_grayscale
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

# greyscale analyse tool. select a folder with svg files. greyscale values output in console.
# ATTENTION: svg-files will be renamed!
# usefull in combination with "_4_3_1_02.pde"
import os

from processing_py import *

shapes = []
file_names = []
dir = None
shape_count = 0
counter = 0


def setup():
    global shapes, file_names, dir, shape_count
    size(256, 256)
    # ------ load shapes ------

    # replace this location with a folder on your machine
    dir = "data"
    if os.path.isdir(dir):
        contents = os.listdir(dir)
        shapes = [None] * len(contents)
        file_names = [None] * len(contents)
        for i in range(len(contents)):
            # skip hidden files and folders starting with a dot, load .png files only
            if contents[i][0] == ".":
                continue
            elif contents[i].lower().endswith(".svg"):
                child_file = os.path.join(dir, contents[i])
                shapes[shape_count] = load_shape(child_file)  # GAP: load_shape() (SVG) is missing
                file_names[shape_count] = contents[i]
                shape_count += 1
    background(1.0)


def draw():
    global counter
    background(1.0)
    shape(shapes[counter], 0, 0, width, height)  # GAP: shape(s, x, y, w, h)
    pixels = load_pixels()

    sum_r = 0
    sum_g = 0
    sum_b = 0

    for i in range(len(pixels)):
        # extract red, green, and blue components from pixels[i]
        r = int(red(pixels[i]) * 255)
        g = int(green(pixels[i]) * 255)
        b = int(blue(pixels[i]) * 255)
        sum_r += r
        sum_g += g
        sum_b += b

    sum_r /= len(pixels)
    sum_g /= len(pixels)
    sum_b /= len(pixels)
    average_grey_scale = sum_r * 0.222 + sum_g * 0.707 + sum_b * 0.071

    print(str(counter) + "  " + file_names[counter] + "  average greyscale -> " + str(round(average_grey_scale)) + "  " + str((1 - average_grey_scale / 255.0) * 100) + " %")

    # rename files
    name_prefix = str(round(average_grey_scale)).zfill(3)
    if not file_names[counter].startswith(name_prefix):
        svg_file = os.path.join(dir, file_names[counter])
        new_name = name_prefix + "_" + file_names[counter]
        os.rename(svg_file, os.path.join(dir, new_name))

    counter += 1
    if counter >= shape_count:
        no_loop()


run()
