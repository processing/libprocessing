# M_2_5_02_TOOL
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

# explore different parameters of drawing lissajous figures
#
# KEYS
# s                   : save png
#
# The controlP5 panel of the original is not ported; its parameters are the
# variables below, change their values to explore the tool.
from datetime import datetime

from processing_py import *

# ------ initial parameters and declarations ------

point_count = 1500  # slider 1 to 8000
lissajous_points = []

freq_x = 13  # slider 1 to 50
freq_y = 11  # slider 1 to 50
phi = 97  # slider 0 to 360

mod_freq_x = 0  # slider 0 to 50
mod_freq_y = 0  # slider 0 to 50

mod_freq_2_x = 11  # slider 0 to 50
mod_freq_2_y = 17  # slider 0 to 50
mod_freq_2_strength = 0.0  # slider 0 to 1

random_offset = 2  # slider 0 to 50

invert_background = False  # toggle
line_weight = 1  # slider 1 to 50
line_alpha = 20  # slider 0 to 100

connect_all_points = True  # toggle
connection_radius = 110  # slider 1 to 500
i1 = 0

min_hue_value = 0  # range slider 0 to 720 (low value)
max_hue_value = 100  # range slider 0 to 720 (high value)
saturation_value = 80  # slider 0 to 100
brightness_value = 0  # slider 0 to 100
invert_hue = False  # toggle

# ------ image output ------

save_one_frame = False


def setup():
    size(800, 800)
    background(1.0)

    calculate_lissajous_points()


def draw():
    global i1, save_one_frame
    # alpha values are in the range 0 to 100
    stroke_weight(line_weight)
    stroke(0.0, line_alpha / 100)
    stroke_cap(ROUND)
    no_fill()

    bg_color = color(1.0)
    if invert_background:
        bg_color = color(0.0)

    # calculate points and start drawing again
    if save_one_frame or i1 == 0:
        calculate_lissajous_points()
        background(bg_color)
        i1 = 0

    if not connect_all_points:
        background(bg_color)

        # simple drawing method
        for i in range(point_count):
            draw_line(lissajous_points[i], lissajous_points[i + 1])
            i1 += 1
    else:
        # drawing method where all points are connected with each other
        # alpha depends on distance of the points

        # draw lines not all at once, just the next 100 milliseconds to keep performance
        draw_end_time = millis() + 100
        if save_one_frame:
            draw_end_time = 2147483647

        while i1 < point_count and millis() < draw_end_time:
            for i2 in range(i1):
                draw_line(lissajous_points[i1], lissajous_points[i2])
            i1 += 1

    # image output
    if save_one_frame:
        save_frame(timestamp() + ".png")
        save_one_frame = False


def calculate_lissajous_points():
    global lissajous_points
    if point_count != len(lissajous_points) - 1:
        lissajous_points = [None] * (point_count + 1)

    random_seed(0)

    for i in range(point_count + 1):
        angle = remap(i, 0, point_count, 0, TWO_PI)

        # an additional modulation of the osscillations
        fmx = sin(angle * mod_freq_2_x) * mod_freq_2_strength + 1
        fmy = sin(angle * mod_freq_2_y) * mod_freq_2_strength + 1

        x = sin(angle * freq_x * fmx + radians(phi)) * cos(angle * mod_freq_x)
        y = sin(angle * freq_y * fmy) * cos(angle * mod_freq_y)

        rx = random(-random_offset, random_offset)
        ry = random(-random_offset, random_offset)

        x = (x * (width / 2 - 30 - random_offset) + width / 2) + rx
        y = (y * (height / 2 - 30 - random_offset) + height / 2) + ry

        lissajous_points[i] = Vec2(x, y)


def draw_line(p1, p2):
    d = dist(p1.x, p1.y, p2.x, p2.y)
    a = pow(1 / (d / connection_radius + 1), 6)

    if d <= connection_radius:
        if not invert_hue:
            h = remap(a, 0, 1, min_hue_value, max_hue_value) % 360
        else:
            h = remap(1 - a, 0, 1, min_hue_value, max_hue_value) % 360
        stroke(hsva(h, saturation_value / 100, brightness_value / 100, (a * line_alpha + (i1 % 2 * 2)) / 100))
        line(p1.x, p1.y, p2.x, p2.y)


def key_pressed():
    global save_one_frame
    if key == "s" or key == "S":
        save_one_frame = True


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
