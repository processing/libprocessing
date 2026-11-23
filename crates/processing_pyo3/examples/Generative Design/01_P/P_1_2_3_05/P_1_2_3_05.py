# P_1_2_3_05
#
# Generative Gestaltung – Creative Coding im Web
# ISBN: 978-3-87439-902-9, First Edition, Hermann Schmidt, Mainz, 2018
# Benedikt Groß, Hartmut Bohnacker, Julia Laub, Claudius Lazzeroni
# with contributions by Joey Lee and Niels Poldervaart
# Copyright 2018
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
# Ported from the p5.js original to processing_py.

# generates a specific color palette and some random 'rect-tilings' with radial gradient
#
# MOUSE
# left click          : new composition
#
# KEYS
# s                   : save png
# c                   : save color palette
import struct
from datetime import datetime

from processing_py import *

color_count = 20
hue_values = [0] * color_count
saturation_values = [0] * color_count
brightness_values = [0] * color_count

act_random_seed = 0
alpha_value = 100


def setup():
    full_screen()
    no_stroke()


def draw():
    no_loop()
    background(0.0)
    random_seed(act_random_seed)

    # ------ colors ------
    # create palette
    for i in range(color_count):
        if i % 2 == 0:
            hue_values[i] = random(180)
            saturation_values[i] = random(50)
            brightness_values[i] = 100
        else:
            hue_values[i] = random(360)
            saturation_values[i] = 100
            brightness_values[i] = random(100)

    # ------ area tiling ------
    # count tiles
    counter = 0
    # row count and row height
    row_count = int(random(5, 30))
    row_height = height / row_count

    # seperate each line in parts
    for i in range(row_count, -1, -1):
        # how many fragments
        part_count = i + 1
        parts = []

        ii = 0
        while ii < part_count:
            # sub fragments or not?
            if random(1.0) < 0.075:
                # take care of big values
                fragments = int(random(2, 20))
                part_count = part_count + fragments
                for iii in range(fragments):
                    parts.append(random(2))
            else:
                parts.append(random(2, 20))
            ii += 1

        # add all subparts
        sum_parts_total = 0
        for ii in range(part_count):
            sum_parts_total += parts[ii]

        # draw rects
        sum_parts_now = 0
        for ii in range(len(parts)):
            sum_parts_now += parts[ii]

            if random(1.0) < 0.45:
                w = remap(parts[ii], 0, sum_parts_total, 0, width)
                h = row_height * 1.5
                px1 = remap(sum_parts_now, 0, sum_parts_total, 0, width)
                px2 = px1 + w
                py1 = row_height * i
                py2 = py1 + h

                index = counter % color_count
                col1 = hsva(hue_values[index], saturation_values[index] / 100, brightness_values[index] / 100, alpha_value / 100)
                # create complementary color
                col2 = hsva(hue_values[index] - 180, saturation_values[index] / 100, brightness_values[index] / 100, alpha_value / 100)
                center_gradient(px1, py1, 0, px2, py2, max(w, h), col1, col2)

            counter += 1


def center_gradient(x1, y1, r1, x2, y2, r2, c1, c2):
    cx = x1 + (x2 - x1) / 2
    cy = y1 + (y2 - y1) / 2
    grd = create_radial_gradient(cx, cy, r1, cx, cy, r2)  # GAP: radial gradient fills (canvas createRadialGradient) are missing
    grd.add_color_stop(0, c1)  # GAP: gradient color stops
    grd.add_color_stop(1, c2)  # GAP: gradient color stops
    fill(grd)  # GAP: fill() with a gradient
    rect(x1, y1, x2 - x1, y2 - y1)


def mouse_released():
    global act_random_seed
    act_random_seed = random(100000)
    loop()


def key_pressed():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == "c" or key == "C":
        # -- save an ase file (adobe swatch export) --
        colors = []
        for i in range(len(hue_values)):
            colors.append(hsva(hue_values[i], saturation_values[i] / 100, brightness_values[i] / 100))
        save_ase(colors, timestamp() + ".ase")


def save_ase(colors, filename):
    # write an ase file (adobe swatch exchange)
    f = open(filename, "wb")
    f.write(b"ASEF")
    f.write(struct.pack(">HHI", 1, 0, len(colors)))
    for c in colors:
        name = c.to_hex()
        block = struct.pack(">H", len(name) + 1) + name.encode("utf-16-be") + b"\x00\x00"
        block = block + b"RGB " + struct.pack(">fff", red(c), green(c), blue(c))
        block = block + struct.pack(">H", 0)
        f.write(struct.pack(">HI", 1, len(block)))
        f.write(block)
    f.close()


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
