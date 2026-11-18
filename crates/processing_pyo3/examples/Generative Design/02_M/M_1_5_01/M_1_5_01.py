# M_1_5_01
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

# how to transform noise values into directions (angles) and brightness levels
#
# MOUSE
# position x/y        : specify noise input range
#
# KEYS
# d                   : toogle display brightness circles on/off
# arrow up            : noise falloff +
# arrow down          : noise falloff -
# arrow left          : noise octaves -
# arrow right         : noise octaves +
# space               : new noise seed
# s                   : save png
from datetime import datetime

from processing_py import *

octaves = 4
falloff = 0.5

arc_color = color(0.0, 0.51, 0.64, 0.39)

tile_size = 40
grid_resolution_x = 0
grid_resolution_y = 0
debug_mode = True
arrow = None


def setup():
    global grid_resolution_x, grid_resolution_y, arrow
    size(800, 800)
    cursor(CROSS)
    grid_resolution_x = round(width / tile_size)
    grid_resolution_y = round(height / tile_size)
    arrow = load_shape("data/arrow.svg")  # GAP: load_shape() (SVG) is missing


def draw():
    background(1.0)

    noise_detail(octaves, falloff)
    noise_x_range = mouse_x / 100.0
    noise_y_range = mouse_y / 100.0

    for g_y in range(grid_resolution_y + 1):
        for g_x in range(grid_resolution_x + 1):
            pos_x = tile_size * g_x
            pos_y = tile_size * g_y

            # get noise value
            noise_x = remap(g_x, 0, grid_resolution_x, 0, noise_x_range)
            noise_y = remap(g_y, 0, grid_resolution_y, 0, noise_y_range)
            noise_value = noise(noise_x, noise_y)
            angle = noise_value * TWO_PI

            push_matrix()
            translate(pos_x, pos_y)

            # debug heatmap
            if debug_mode:
                no_stroke()
                ellipse_mode(CENTER)
                fill(noise_value)
                ellipse(0, 0, tile_size * 0.25, tile_size * 0.25)

            # arc
            no_fill()
            stroke_cap(SQUARE)
            stroke_weight(1)
            stroke(arc_color)
            arc(0, 0, tile_size * 0.75, tile_size * 0.75, 0, angle)

            # arrow
            stroke(0.0)
            stroke_weight(0.75)
            rotate(angle)
            shape(arrow, 0, 0, tile_size * 0.75, tile_size * 0.75)  # GAP: shape(s, x, y, w, h)
            pop_matrix()

    print("octaves: " + str(octaves) + " falloff: " + str(falloff) + " noiseXRange: 0-" + str(noise_x_range) + " noiseYRange: 0-" + str(noise_y_range))


def key_released():
    global debug_mode
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == "d" or key == "D":
        debug_mode = not debug_mode
    if key == " ":
        noise_seed(int(random(100000)))


def key_pressed():
    global falloff, octaves
    if key_code == UP:
        falloff += 0.05
    if key_code == DOWN:
        falloff -= 0.05
    if falloff > 1.0:
        falloff = 1.0
    if falloff < 0.0:
        falloff = 0.0

    if key_code == LEFT_ARROW:
        octaves -= 1
    if key_code == RIGHT_ARROW:
        octaves += 1
    if octaves < 0:
        octaves = 0


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
