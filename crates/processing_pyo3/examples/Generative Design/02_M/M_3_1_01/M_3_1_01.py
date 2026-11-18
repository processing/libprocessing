# M_3_1_01
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

# draws a flat grid
#
# MOUSE
# click + drag        : rotate
from datetime import datetime

from processing_py import *

# number of grid points horizontal
x_count = 4
# number of grid points vertical
y_count = 4

# variables for rotation
offset_x = 0
offset_y = 0
click_x = 0
click_y = 0
rotation_x = 0
rotation_y = 0
target_rotation_x = 0
target_rotation_y = 0
click_rotation_x = 0
click_rotation_y = 0


def setup():
    size(400, 400)
    mode_3d()

    fill(1.0)
    stroke_weight(1 / 80.0)


def draw():
    background(1.0)

    set_view()

    scale(40)

    # Draw Mesh
    for y in range(y_count):
        begin_shape(QUAD_STRIP)
        for x in range(x_count + 1):
            vertex(x, y, 0)  # GAP: vertex(x, y, z) in immediate mode
            vertex(x, y + 1, 0)  # GAP: vertex(x, y, z) in immediate mode
        end_shape()


def mouse_pressed():
    global click_x, click_y, click_rotation_x, click_rotation_y
    click_x = mouse_x
    click_y = mouse_y
    click_rotation_x = rotation_x
    click_rotation_y = rotation_y


def set_view():
    global offset_x, offset_y, target_rotation_x, target_rotation_y, rotation_x, rotation_y
    # translate(width * 0.5, height * 0.5) is not needed: the 3D origin is the canvas center

    if mouse_is_pressed:
        offset_x = mouse_x - click_x
        offset_y = mouse_y - click_y
        target_rotation_x = click_rotation_x + offset_x / width * TWO_PI
        target_rotation_y = min(max(click_rotation_y + offset_y / height * TWO_PI, -HALF_PI), HALF_PI)
        rotation_x += (target_rotation_x - rotation_x) * 0.25
        rotation_y += (target_rotation_y - rotation_y) * 0.25
    rotate_x(-rotation_y)
    rotate_y(rotation_x)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
