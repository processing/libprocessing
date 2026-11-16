# P_4_0_01
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

# draw a grid of streched copies of an image
#
# MOUSE
# position x           : tile count horizontally
# position y           : tile count vertically
#
# KEYS
# s                    : save png
from datetime import datetime

from processing_py import *

img = None


def setup():
    global img
    size(650, 450)
    img = load_image("data/image.jpg")


def draw():
    tile_count_x = int(mouse_x) // 3 + 1
    tile_count_y = int(mouse_y) // 3 + 1
    step_x = width / tile_count_x
    step_y = height / tile_count_y
    grid_y = 0.0
    while grid_y < height:
        grid_x = 0.0
        while grid_x < width:
            image(img, grid_x, grid_y, step_x, step_y)
            grid_x += step_x
        grid_y += step_y


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
