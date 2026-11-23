# P_4_1_1_01
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

# cutting and multiplying an area of the image
#
# MOUSE
# position x/y         : area position
# left click           : multiply the area
#
# KEYS
# 1-3                  : area size
# r                    : toggle random area
# s                    : save png
from datetime import datetime

from processing_py import *

img = None

tile_count_x = 4
tile_count_y = 4
tile_count = tile_count_x * tile_count_y
image_tiles = [None] * tile_count

tile_width = 0
tile_height = 0

crop_x = 0
crop_y = 0

select_mode = True
random_mode = False


def setup():
    global img, tile_width, tile_height
    size(1600, 1200)
    img = load_image("data/image.jpg")
    image(img, 0, 0)
    no_cursor()

    tile_width = width // tile_count_y
    tile_height = height // tile_count_x


def draw():
    global crop_x, crop_y
    if select_mode == True:
        # in selection mode, a white selection rectangle is drawn over the image
        crop_x = constrain(int(mouse_x), 0, width - tile_width)
        crop_y = constrain(int(mouse_y), 0, height - tile_height)
        image(img, 0, 0)
        no_fill()
        stroke(1.0)
        rect(crop_x, crop_y, tile_width, tile_height)
    else:
        # reassemble image
        i = 0
        for grid_y in range(tile_count_y):
            for grid_x in range(tile_count_x):
                image(image_tiles[i], grid_x * tile_width, grid_y * tile_height)
                i += 1


def crop_tiles():
    global tile_width, tile_height, tile_count, image_tiles, crop_x, crop_y
    tile_width = width // tile_count_y
    tile_height = height // tile_count_x
    tile_count = tile_count_x * tile_count_y
    image_tiles = [None] * tile_count

    i = 0
    for grid_y in range(tile_count_y):
        for grid_x in range(tile_count_x):
            if random_mode:
                crop_x = int(random(mouse_x - tile_width / 2, mouse_x + tile_width / 2))
                crop_y = int(random(mouse_y - tile_height / 2, mouse_y + tile_height / 2))
            crop_x = constrain(crop_x, 0, width - tile_width)
            crop_y = constrain(crop_y, 0, height - tile_height)
            image_tiles[i] = img.get(crop_x, crop_y, tile_width, tile_height)  # GAP: Image.get(x, y, w, h) (cut out a sub-image) is missing; get() only reads one pixel
            i += 1


def mouse_moved():
    global select_mode
    select_mode = True


def mouse_released():
    global select_mode
    select_mode = False
    crop_tiles()


def key_released():
    global random_mode, tile_count_x, tile_count_y
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "r" or key == "R":
        random_mode = not random_mode
        crop_tiles()

    if key == "1":
        tile_count_y = 4
        tile_count_x = 4
        crop_tiles()
    if key == "2":
        tile_count_y = 10
        tile_count_x = 10
        crop_tiles()
    if key == "3":
        tile_count_y = 20
        tile_count_x = 20
        crop_tiles()


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
