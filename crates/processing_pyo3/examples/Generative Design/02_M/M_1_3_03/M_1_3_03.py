# M_1_3_03
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

# creates a texture based on noise values
#
# MOUSE
# position x/y        : specify noise input range
#
# KEYS
# 1-2                 : set noise mode
# arrow up            : noise falloff +
# arrow down          : noise falloff -
# arrow left          : noise octaves -
# arrow right         : noise octaves +
# s                   : save png
from datetime import datetime

from processing_py import *

octaves = 4
falloff = 0.5

noise_mode = 1


def setup():
    size(512, 512)
    pixel_density(1)
    cursor(CROSS)


def draw():
    background(0.0)

    noise_detail(octaves, falloff)

    noise_x_range = int(mouse_x / 10)
    noise_y_range = int(mouse_y / 10)

    px = load_pixels()
    for x in range(width):
        for y in range(height):
            noise_x = remap(x, 0, width, 0, noise_x_range)
            noise_y = remap(y, 0, height, 0, noise_y_range)

            noise_value = 0
            if noise_mode == 1:
                noise_value = noise(noise_x, noise_y) * 255
            elif noise_mode == 2:
                n = noise(noise_x, noise_y) * 24
                noise_value = (n - int(n)) * 255

            px[x + y * width] = color(noise_value / 255)
    update_pixels(px)

    print("octaves: " + str(octaves) + " falloff: " + str(falloff) + " noiseXRange: 0-" + str(noise_x_range) + " noiseYRange: 0-" + str(noise_y_range))


def key_released():
    global noise_mode
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == " ":
        noise_seed(int(random(100000)))
    if key == "1":
        noise_mode = 1
    if key == "2":
        noise_mode = 2


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


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
