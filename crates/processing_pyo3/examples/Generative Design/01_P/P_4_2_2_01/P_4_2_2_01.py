# P_4_2_2_01
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

# simple overview of a video file.
#
# KEYS
# s                  : save png
from datetime import datetime

from processing_py import *

movie = None

# horizontal and vertical grid count
# take care of the aspect ratio ... here 4:3
tile_count_x = 12
tile_count_y = 16
tile_width = 0
tile_height = 0
image_count = tile_count_x * tile_count_y
current_image = 0
grid_x = 0
grid_y = 0


def setup():
    global movie, tile_width, tile_height
    size(1024, 1024)
    background(0.0)

    # specify a path to load a video
    # or simply put it into the data folder
    path = "your/path/to/video"
    movie = load_movie(path)  # GAP: no video playback; load_movie(path) -> Movie is missing
    movie.play()  # GAP: Movie.play()

    tile_width = width / tile_count_x
    tile_height = height / tile_count_y


def draw():
    global grid_x, grid_y, current_image
    pos_x = tile_width * grid_x
    pos_y = tile_height * grid_y

    # calculate the current time in movieclip
    movie_pos = remap(current_image, 0, image_count, 0, movie.duration())  # GAP: Movie.duration()
    movie.jump(movie_pos)  # GAP: Movie.jump(seconds)
    movie.read()  # GAP: Movie.read()
    image(movie, pos_x, pos_y, tile_width, tile_height)  # GAP: image() with a Movie source

    # new grid position
    grid_x += 1
    if grid_x >= tile_count_x:
        grid_x = 0
        grid_y += 1

    current_image += 1
    if current_image >= image_count:
        no_loop()


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
