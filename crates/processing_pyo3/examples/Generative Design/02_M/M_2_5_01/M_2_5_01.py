# M_2_5_01
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

# draw lissajous figures with all points connected
#
# KEYS
# 1/2               : frequency x -/+
# 3/4               : frequency y -/+
# arrow left/right  : phi -/+
# 7/8               : modulation frequency x -/+
# 9/0               : modulation frequency y -/+
# s                 : save png
from datetime import datetime

from processing_py import *

# ------ initial parameters and declarations ------

point_count = 1000
lissajous_points = []

freq_x = 4
freq_y = 7
phi = 15

mod_freq_x = 3
mod_freq_y = 2

line_weight = 1
line_color = color(0.0)
# line_color = color(0.0, 0.51, 0.64)
line_alpha = 50

connection_radius = 100
connection_ramp = 6


def setup():
    size(800, 800)

    calculate_lissajous_points()
    draw_lissajous()


def draw():
    pass


def calculate_lissajous_points():
    global lissajous_points
    lissajous_points = []

    for i in range(point_count + 1):
        angle = remap(i, 0, point_count, 0, TWO_PI)

        x = sin(angle * freq_x + radians(phi)) * cos(angle * mod_freq_x)
        y = sin(angle * freq_y) * cos(angle * mod_freq_y)

        x = x * (width / 2 - 30)
        y = y * (height / 2 - 30)

        lissajous_points.append(Vec2(x, y))


def draw_lissajous():
    # alpha values are in the range 0 to 100
    background(1.0)
    stroke_weight(line_weight)
    stroke_cap(ROUND)
    no_fill()

    push_matrix()
    translate(width / 2, height / 2)

    for i1 in range(point_count):
        for i2 in range(i1):
            p1 = lissajous_points[i1]
            p2 = lissajous_points[i2]

            d = dist(p1.x, p1.y, p2.x, p2.y)
            a = pow(1 / (d / connection_radius + 1), 6)

            if d <= connection_radius:
                stroke(line_color.with_alpha(a * line_alpha / 100))
                line(p1.x, p1.y, p2.x, p2.y)
    pop_matrix()


def key_pressed():
    global freq_x, freq_y, phi, mod_freq_x, mod_freq_y
    if key == "s" or key == "S":
        save_frame(timestamp() + ".png")

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

    calculate_lissajous_points()
    draw_lissajous()

    print("freqX: " + str(freq_x) + ", freqY: " + str(freq_y) + ", phi: " + str(phi) + ", modFreqX: " + str(mod_freq_x) + ", modFreqY: " + str(mod_freq_y))


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
