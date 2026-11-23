# P_2_2_4_02
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

# limited diffusion aggregation
#
# KEYS
# 1             : toggle draw original position of circles
# s             : save png
from datetime import datetime

from processing_py import *

max_count = 5000  # max count of the cirlces
current_count = 1
newx = [0.0] * max_count
newy = [0.0] * max_count
x = [0.0] * max_count
y = [0.0] * max_count
r = [0.0] * max_count  # radius

draw_ghosts = False


def setup():
    size(800, 800)

    # first circle
    x[0] = width / 2
    y[0] = height / 2
    # r[0] = 10
    r[0] = 360


def draw():
    global current_count
    background(1.0)

    stroke_weight(0.5)
    # no_fill()

    # create a random set of parameters
    new_r = random(1, 7)
    new_x = random(0 + new_r, width - new_r)
    new_y = random(0 + new_r, height - new_r)

    closest_dist = 100000000
    closest_index = 0
    # which circle is the closest?
    for i in range(current_count):
        new_dist = dist(new_x, new_y, x[i], y[i])
        if new_dist < closest_dist:
            closest_dist = new_dist
            closest_index = i

    # aline it to the closest circle outline
    angle = atan2(new_y - y[closest_index], new_x - x[closest_index])

    newx[current_count] = new_x
    newy[current_count] = new_y
    x[current_count] = x[closest_index] + cos(angle) * (r[closest_index] + new_r)
    y[current_count] = y[closest_index] + sin(angle) * (r[closest_index] + new_r)
    r[current_count] = new_r
    current_count += 1

    # draw circles at random position and lines
    if draw_ghosts:
        for i in range(1, current_count):
            fill(0.9)
            ellipse(newx[i], newy[i], r[i] * 2, r[i] * 2)
            line(newx[i], newy[i], x[i], y[i])

    for i in range(current_count):
        if i == 0:
            no_fill()
        else:
            fill(0.2)
        ellipse(x[i], y[i], r[i] * 2, r[i] * 2)

    if current_count >= max_count:
        no_loop()


def key_released():
    global draw_ghosts
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        draw_ghosts = not draw_ghosts


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
