# P_2_2_2_01
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

# draw the path of an intelligent agent
#
# MOUSE
# position x          : composition speed of the picture
#
# KEYS
# Delete/Backspace    : clear display
# s                   : save png
from datetime import datetime

from processing_py import *

NORTH = 0
EAST = 1
SOUTH = 2
WEST = 3

pos_x = 0
pos_y = 0
pos_x_cross = 0
pos_y_cross = 0

direction = SOUTH
angle_count = 7  # 1 - ...
angle = 0
step_size = 3
min_length = 10


def setup():
    global angle, pos_x, pos_y, pos_x_cross, pos_y_cross
    size(600, 600)
    background(1.0)

    angle = get_random_angle(direction)

    pos_x = int(random(0, width))
    pos_y = 5
    pos_x_cross = pos_x
    pos_y_cross = pos_y


def draw():
    global angle, direction, pos_x, pos_y, pos_x_cross, pos_y_cross
    for i in range(int(mouse_x) + 1):

        # ------ draw dot at current position ------
        stroke_weight(1)
        stroke(0.5)
        point(pos_x, pos_y)

        # ------ make step ------
        pos_x += cos(radians(angle)) * step_size
        pos_y += sin(radians(angle)) * step_size

        # ------ check if agent is near one of the display borders ------
        reached_border = False

        if pos_y <= 5:
            direction = SOUTH
            reached_border = True
        elif pos_x >= width - 5:
            direction = WEST
            reached_border = True
        elif pos_y >= height - 5:
            direction = NORTH
            reached_border = True
        elif pos_x <= 5:
            direction = EAST
            reached_border = True

        # ------ if agent is crossing his path or border was reached ------
        px = int(pos_x)
        py = int(pos_y)
        if get(px, py) != color(1.0) or reached_border:  # GAP: get() reads the white background back as 0.99999994, which never equals color(1.0), so the agent turns on every step
            angle = get_random_angle(direction)
            distance = dist(pos_x, pos_y, pos_x_cross, pos_y_cross)
            if distance >= min_length:
                stroke_weight(3)
                stroke(0.0)
                line(pos_x, pos_y, pos_x_cross, pos_y_cross)
            pos_x_cross = pos_x
            pos_y_cross = pos_y


def get_random_angle(the_direction):
    a = (floor(random(-angle_count, angle_count)) + 0.5) * 90.0 / angle_count

    if the_direction == NORTH:
        return a - 90
    if the_direction == EAST:
        return a
    if the_direction == SOUTH:
        return a + 90
    if the_direction == WEST:
        return a + 180
    return 0


def key_released():
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == " ":
        no_loop()


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
