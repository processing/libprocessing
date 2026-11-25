# P_4_3_1_01
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

# pixel mapping. each pixel is translated into a new element
#
# MOUSE
# position x/y        : various parameters (depending on draw mode)
#
# KEYS
# 1-9                 : switch draw mode
# s                   : save png
from datetime import datetime

from processing_py import *

img = None
draw_mode = 1


def setup():
    global img
    size(603, 873)  # size should be multiple of img width and height
    img = load_image("data/pic.png")
    print(str(img.width) + " x " + str(img.height))


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

            if draw_mode == 1:
                # greyscale to stroke weight
                w1 = remap(greyscale, 0, 255, 15, 0.1)
                stroke(0.0)
                stroke_weight(w1 * mouse_x_factor)
                line(pos_x, pos_y, pos_x + 5, pos_y + 5)
            elif draw_mode == 2:
                # greyscale to ellipse area
                fill(0.0)
                no_stroke()
                r2 = 1.1284 * sqrt(tile_width * tile_width * (1 - greyscale / 255.0))
                r2 = r2 * mouse_x_factor * 3
                ellipse(pos_x, pos_y, r2, r2)
            elif draw_mode == 3:
                # greyscale to line length
                l3 = remap(greyscale, 0, 255, 30, 0.1)
                l3 = l3 * mouse_x_factor
                stroke(0.0)
                stroke_weight(10 * mouse_y_factor)
                line(pos_x, pos_y, pos_x + l3, pos_y + l3)
            elif draw_mode == 4:
                # greyscale to rotation, line length and stroke weight
                stroke(0.0)
                w4 = remap(greyscale, 0, 255, 10, 0)
                stroke_weight(w4 * mouse_x_factor + 0.1)
                l4 = remap(greyscale, 0, 255, 35, 0)
                l4 = l4 * mouse_y_factor
                push_matrix()
                translate(pos_x, pos_y)
                rotate(greyscale / 255.0 * PI)
                line(0, 0, 0 + l4, 0 + l4)
                pop_matrix()
            elif draw_mode == 5:
                # greyscale to line relief
                w5 = remap(greyscale, 0, 255, 5, 0.2)
                stroke_weight(w5 * mouse_y_factor + 0.1)
                # get neighbour pixel, limit it to image width
                c2 = img.get(min(grid_x + 1, img.width - 1), grid_y)
                stroke(c2)
                greyscale2 = int(red(c2) * 255 * 0.222 + green(c2) * 255 * 0.707 + blue(c2) * 255 * 0.071)
                h5 = 50 * mouse_x_factor
                d1 = remap(greyscale, 0, 255, h5, 0)
                d2 = remap(greyscale2, 0, 255, h5, 0)
                line(pos_x - d1, pos_y + d1, pos_x + tile_width - d2, pos_y + d2)
            elif draw_mode == 6:
                # pixel color to fill, greyscale to ellipse size
                w6 = remap(greyscale, 0, 255, 25, 0)
                no_stroke()
                fill(c)
                ellipse(pos_x, pos_y, w6 * mouse_x_factor, w6 * mouse_x_factor)
            elif draw_mode == 7:
                stroke(c)
                w7 = remap(greyscale, 0, 255, 5, 0.1)
                stroke_weight(w7)
                fill(1.0, mouse_x_factor)
                push_matrix()
                translate(pos_x, pos_y)
                rotate(greyscale / 255.0 * PI * mouse_y_factor)
                rect(0, 0, 15, 15)
                pop_matrix()
            elif draw_mode == 8:
                no_stroke()
                fill(greyscale / 255, greyscale / 255 * mouse_x_factor, mouse_y_factor)
                rect(pos_x, pos_y, 3.5, 3.5)
                rect(pos_x + 4, pos_y, 3.5, 3.5)
                rect(pos_x, pos_y + 4, 3.5, 3.5)
                rect(pos_x + 4, pos_y + 4, 3.5, 3.5)
            elif draw_mode == 9:
                stroke(1.0, greyscale / 255, 0.0)
                no_fill()
                push_matrix()
                translate(pos_x, pos_y)
                rotate(greyscale / 255.0 * PI)
                stroke_weight(1)
                rect(0, 0, 15 * mouse_x_factor, 15 * mouse_y_factor)
                w9 = remap(greyscale, 0, 255, 15, 0.1)
                stroke_weight(w9)
                stroke(0.0, 70 / 255)
                ellipse(0, 0, 10, 5)
                pop_matrix()


def key_released():
    global draw_mode
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        draw_mode = 1
    if key == "2":
        draw_mode = 2
    if key == "3":
        draw_mode = 3
    if key == "4":
        draw_mode = 4
    if key == "5":
        draw_mode = 5
    if key == "6":
        draw_mode = 6
    if key == "7":
        draw_mode = 7
    if key == "8":
        draw_mode = 8
    if key == "9":
        draw_mode = 9


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
