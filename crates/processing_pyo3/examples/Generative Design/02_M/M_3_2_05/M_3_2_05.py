# M_3_2_05
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

# optimized code for calculating and drawing a mesh.
#
# MOUSE
# click + drag        : rotate
from datetime import datetime

from processing_py import *

# grid definition horizontal
u_count = 30
u_min = 0
u_max = 5

# grid definition vertical
v_count = 30
v_min = -1
v_max = 1

# array for the grid points
# initialize array
points = []

# view rotation
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
    size(800, 800)
    mode_3d()
    fill(1.0)
    stroke_weight(1 / 400.0)

    # fill array
    for iv in range(v_count + 1):
        points.append([])
        for iu in range(u_count + 1):
            u = remap(iu, 0, u_count, u_min, u_max)
            v = remap(iv, 0, v_count, v_min, v_max)

            points[iv].append(Vec3(0, 0, 0))
            points[iv][iu].x = v
            points[iv][iu].y = sin(u) * cos(v)
            points[iv][iu].z = cos(u) * cos(v)


def draw():
    background(1.0)

    set_view()

    scale(200)

    # draw mesh
    for iv in range(v_count):
        begin_shape(QUAD_STRIP)
        for iu in range(u_count + 1):
            vertex(points[iv][iu].x, points[iv][iu].y, points[iv][iu].z)  # GAP: vertex(x, y, z) in immediate mode
            vertex(points[iv + 1][iu].x, points[iv + 1][iu].y, points[iv + 1][iu].z)  # GAP: vertex(x, y, z) in immediate mode
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
