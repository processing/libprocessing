# P_2_2_5_01
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

# pack as many cirlces as possible together
#
# MOUSE
# press + position x/y : move area of interest
#
# KEYS
# arrow up/down        : resize area of interest
# s                    : save png
from datetime import datetime

from processing_py import *

max_count = 5000  # max count of the cirlces
current_count = 1
x = [0.0] * max_count
y = [0.0] * max_count
r = [0.0] * max_count  # radius
closest_index = [0] * max_count  # index

min_radius = 3
max_radius = 50

# for mouse and arrow up/down interaction
mouse_rect = 30


def setup():
    size(800, 800)
    no_fill()
    cursor(CROSS)

    # first circle
    x[0] = 200
    y[0] = 100
    r[0] = 50
    closest_index[0] = 0


def draw():
    global current_count
    background(1.0)

    # create a random position
    new_x = random(0 + max_radius, width - max_radius)
    new_y = random(0 + max_radius, height - max_radius)
    new_r = min_radius

    # create a random position according to mouse position
    if mouse_is_pressed == True:
        new_x = random(mouse_x - mouse_rect / 2, mouse_x + mouse_rect / 2)
        new_y = random(mouse_y - mouse_rect / 2, mouse_y + mouse_rect / 2)
        new_r = 1

    intersection = False

    # find out, if new circle intersects with one of the others
    for i in range(current_count):
        d = dist(new_x, new_y, x[i], y[i])
        if d < (new_r + r[i]):
            intersection = True
            break

    # no intersection ... add a new circle
    if intersection == False:
        # get closest neighbour and closest possible radius
        new_radius = width
        for i in range(current_count):
            d = dist(new_x, new_y, x[i], y[i])
            if new_radius > d - r[i]:
                new_radius = d - r[i]
                closest_index[current_count] = i

        if new_radius > max_radius:
            new_radius = max_radius

        x[current_count] = new_x
        y[current_count] = new_y
        r[current_count] = new_radius
        current_count += 1

    # draw them
    for i in range(current_count):
        stroke(0.0)
        stroke_weight(1.5)
        ellipse(x[i], y[i], r[i] * 2, r[i] * 2)
        stroke(226 / 255, 185 / 255, 0.0)
        stroke_weight(0.75)
        n = closest_index[i]
        line(x[i], y[i], x[n], y[n])

    # visualize the random range of the new positions
    if mouse_is_pressed == True:
        stroke(1.0, 200 / 255, 0.0)
        stroke_weight(2)
        rect(mouse_x - mouse_rect / 2, mouse_y - mouse_rect / 2, mouse_rect, mouse_rect)

    if current_count >= max_count:
        no_loop()


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


def key_pressed():
    global mouse_rect
    # mouse_rect ctrls arrowkeys up/down
    if key_code == UP:
        mouse_rect += 4
    if key_code == DOWN:
        mouse_rect -= 4


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
