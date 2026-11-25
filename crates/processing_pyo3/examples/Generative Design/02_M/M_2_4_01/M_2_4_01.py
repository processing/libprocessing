# M_2_4_01
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

# draws a three dimensional lissajous curve
#
# MOUSE
# right click + drag  : camera controls
#
# KEYS
# 1/2                 : frequency x -/+
# 3/4                 : frequency y -/+
# 5/6                 : frequency z -/+
# arrow left/right    : phi x -/+
# arrow down/up       : phi y -/+
# s                   : save png
#
# The high resolution export (TileSaver, key p) of the original is not ported.
from processing_py import *

point_count = 600
freq_x = 1
freq_y = 4
freq_z = 2
phi_x = 0
phi_y = 0

lissajous_points = []

# the triangles and the path are kept as geometries, rebuilt whenever the points change
triangles = None
path = None

# ------ mouse interaction ------
offset_x = 0
offset_y = 0
click_x = 0
click_y = 0
zoom = -400
rotation_x = 0
rotation_y = 0
target_rotation_x = 0
target_rotation_y = 0
click_rotation_x = 0
click_rotation_y = 0


def setup():
    size(600, 600)
    mode_3d()
    roughness(0.6)

    calculate_lissajous_points()


def draw():
    global offset_x, offset_y, target_rotation_x, target_rotation_y, rotation_x, rotation_y
    background(1.0)

    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))  # GAP: lights() is missing (ambient + directional, lighting both sides); with this single light the faces turned away from it render black

    # ------ set view ------
    translate(0, 0, zoom)

    if mouse_is_pressed and mouse_button == RIGHT:
        offset_x = mouse_x - click_x
        offset_y = mouse_y - click_y

        target_rotation_x = min(max(click_rotation_x + offset_y / width * TWO_PI, -HALF_PI), HALF_PI)
        target_rotation_y = click_rotation_y + offset_x / height * TWO_PI

    rotation_x += (target_rotation_x - rotation_x) * 0.25
    rotation_y += (target_rotation_y - rotation_y) * 0.25
    rotate_x(-rotation_x)
    rotate_y(rotation_y)

    # ------ draw triangles ------
    no_stroke()
    fill(1.0)
    draw_geometry(triangles)

    # ------ draw lissajous path ------
    stroke(0.0)
    stroke_weight(1)
    no_fill()
    draw_geometry(path)


def calculate_lissajous_points():
    global lissajous_points, triangles, path
    lissajous_points = []
    f = width / 2

    for i in range(point_count + 1):
        angle = remap(i, 0, point_count, 0, TWO_PI)
        x = sin(angle * freq_x + radians(phi_x)) * sin(angle * 2) * f
        y = sin(angle * freq_y + radians(phi_y)) * f
        z = sin(angle * freq_z) * f
        lissajous_points.append(Vec3(x, y, z))

    # ------ triangles ------
    triangles = create_geometry(topology=TRIANGLES)
    for i in range(point_count - 2):
        if i % 3 == 0:
            # lit geometry needs a normal per vertex
            n = lissajous_points[i].cross(lissajous_points[i + 2])
            n.normalize()
            triangles.normal(n.x, n.y, n.z)
            # gradient for every trinangle to lissajou path
            triangles.color(50 / 255, 50 / 255, 50 / 255, 1.0)
            triangles.vertex(0, 0, 0)
            triangles.color(200 / 255, 200 / 255, 200 / 255, 1.0)
            triangles.vertex(lissajous_points[i].x, lissajous_points[i].y, lissajous_points[i].z)
            triangles.vertex(lissajous_points[i + 2].x, lissajous_points[i + 2].y, lissajous_points[i + 2].z)

    # ------ lissajous path ------
    path = create_geometry(topology=LINE_STRIP)
    for i in range(point_count + 1):
        path.color(1.0, 1.0, 1.0, 1.0)
        path.vertex(lissajous_points[i].x, lissajous_points[i].y, lissajous_points[i].z)


def key_pressed():
    global freq_x, freq_y, freq_z, phi_x, phi_y
    if key == "s" or key == "S":
        save_frame(str(freq_x) + "_" + str(freq_y) + "_" + str(freq_z) + "_" + str(int(phi_x)) + "_" + str(int(phi_y)) + ".png")

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

    if key == "5":
        freq_z -= 1
    if key == "6":
        freq_z += 1
    freq_z = max(freq_z, 1)

    if key_code == LEFT_ARROW:
        phi_x -= 15
    if key_code == RIGHT_ARROW:
        phi_x += 15

    if key_code == DOWN:
        phi_y -= 15
    if key_code == UP:
        phi_y += 15

    calculate_lissajous_points()

    print("freqX: " + str(freq_x) + ", freqY: " + str(freq_y) + ", freqZ: " + str(freq_z) + ", phiX: " + str(phi_x) + ", phiY: " + str(phi_y))


def mouse_pressed():
    global click_x, click_y, click_rotation_x, click_rotation_y
    click_x = mouse_x
    click_y = mouse_y
    click_rotation_x = rotation_x
    click_rotation_y = rotation_y


run()
