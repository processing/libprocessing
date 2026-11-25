# P_2_2_1_01
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

# draw the path of a stupid agent
#
# MOUSE
# position x          : drawing speed
#
# KEYS
# BACKSPACE           : delete display
# s                   : save png
from datetime import datetime

from processing_py import *

NORTH = 0
NORTHEAST = 1
EAST = 2
SOUTHEAST = 3
SOUTH = 4
SOUTHWEST = 5
WEST = 6
NORTHWEST = 7

step_size = 1
diameter = 1

direction = 0
pos_x = 0
pos_y = 0


def setup():
    global pos_x, pos_y
    size(800, 800)
    background(1.0)
    no_stroke()
    pos_x = width / 2
    pos_y = height / 2


def draw():
    global direction, pos_x, pos_y
    for i in range(int(mouse_x) + 1):
        direction = int(random(0, 8))

        if direction == NORTH:
            pos_y -= step_size
        elif direction == NORTHEAST:
            pos_x += step_size
            pos_y -= step_size
        elif direction == EAST:
            pos_x += step_size
        elif direction == SOUTHEAST:
            pos_x += step_size
            pos_y += step_size
        elif direction == SOUTH:
            pos_y += step_size
        elif direction == SOUTHWEST:
            pos_x -= step_size
            pos_y += step_size
        elif direction == WEST:
            pos_x -= step_size
        elif direction == NORTHWEST:
            pos_x -= step_size
            pos_y -= step_size

        if pos_x > width:
            pos_x = 0
        if pos_x < 0:
            pos_x = width
        if pos_y < 0:
            pos_y = height
        if pos_y > height:
            pos_y = 0

        fill(0.0, 0.16)
        ellipse(pos_x + step_size / 2, pos_y + step_size / 2, diameter, diameter)


def key_released():
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
