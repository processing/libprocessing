# M_2_3_02
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

# draws a modulated lissajous curve
#
# MOUSE
# position x        : number of points
#
# KEYS
# d                 : draw mode
# 1/2               : frequency x -/+
# 3/4               : frequency y -/+
# arrow left/right  : phi -/+
# 7/8               : modulation frequency x -/+
# 9/0               : modulation frequency y -/+
# s                 : save png
from datetime import datetime

from processing_py import *

point_count = 500
freq_x = 1
freq_y = 4
phi = 60

mod_freq_x = 2
mod_freq_y = 1
modulation_phi = 0

angle = 0
x = 0
y = 0
w = 0
max_dist = 0
old_x = 0
old_y = 0

draw_mode = 2


def setup():
    global max_dist
    size(600, 600)
    stroke_cap(ROUND)

    max_dist = sqrt(sq(width / 2 - 50) + sq(height / 2 - 50))


def draw():
    global point_count, angle, x, y, w, old_x, old_y
    background(1.0)

    translate(width / 2, height / 2)

    point_count = int(mouse_x) * 2 + 200

    if draw_mode == 1:
        stroke(0.0)
        stroke_weight(1)

        begin_shape()
        for i in range(point_count + 1):
            angle = remap(i, 0, point_count, 0, TWO_PI)
            x = sin(angle * freq_x + radians(phi)) * cos(angle * mod_freq_x)
            y = sin(angle * freq_y) * cos(angle * mod_freq_y)
            x = x * (width / 2 - 50)
            y = y * (height / 2 - 50)
            vertex(x, y)
        end_shape()

    elif draw_mode == 2:
        stroke_weight(8)

        for i in range(point_count + 1):
            angle = remap(i, 0, point_count, 0, TWO_PI)

            # amplitude modulation
            x = sin(angle * freq_x + radians(phi)) * cos(angle * mod_freq_x)
            y = sin(angle * freq_y) * cos(angle * mod_freq_y)

            x = x * (width / 2 - 50)
            y = y * (height / 2 - 50)

            if i > 0:
                w = dist(x, y, 0, 0)
                line_alpha = remap(w, 0, max_dist, 255, 0)
                stroke(i % 2 * 2 / 255, line_alpha / 255)
                line(old_x, old_y, x, y)

            old_x = x
            old_y = y


def key_pressed():
    global draw_mode, freq_x, freq_y, phi, mod_freq_x, mod_freq_y
    if key == "s" or key == "S":
        save_frame(timestamp() + ".png")

    if key == "d" or key == "D":
        if draw_mode == 1:
            draw_mode = 2
        else:
            draw_mode = 1

    if key == "1":
        freq_x -= 1
    if key == "2":
        freq_x += 1
    freq_x = max(freq_x, 1)

    if key == "3":
        freq_y -= 1
    if key == "4":
        freq_y += 1
    freq_y = max(freq_y, 1)

    if key_code == LEFT_ARROW:
        phi -= 15
    if key_code == RIGHT_ARROW:
        phi += 15

    if key == "7":
        mod_freq_x -= 1
    if key == "8":
        mod_freq_x += 1
    mod_freq_x = max(mod_freq_x, 1)

    if key == "9":
        mod_freq_y -= 1
    if key == "0":
        mod_freq_y += 1
    mod_freq_y = max(mod_freq_y, 1)

    print("freqX: " + str(freq_x) + ", freqY: " + str(freq_y) + ", phi: " + str(phi) + ", modFreqX: " + str(mod_freq_x) + ", modFreqY: " + str(mod_freq_y))


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
