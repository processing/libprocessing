# P_1_2_3_02
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

# generates a specific color palette and some random "rect-tilings"
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


def setup():
    size(800, 800)
    no_stroke()


def draw():
    # ------ colors ------
    # create palette
    for i in range(color_count):
        if i % 2 == 0:
            hue_values[i] = int(random(0, 360))
            saturation_values[i] = 100
            brightness_values[i] = int(random(0, 100))
        else:
            hue_values[i] = 195
            saturation_values[i] = int(random(0, 100))
            brightness_values[i] = 100

    # ------ area tiling ------
    # count tiles
    counter = 0
    # row count and row height
    row_count = int(random(5, 40))
    row_height = height / row_count

    for i in range(row_count):
        # seperate each line in parts
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
            # get component color values
            index = counter % color_count
            fill(hsva(hue_values[index], saturation_values[index] / 100, brightness_values[index] / 100))

            sum_parts_now += parts[ii]
            rect(remap(sum_parts_now, 0, sum_parts_total, 0, width), row_height * i,
                 remap(parts[ii], 0, sum_parts_total, 0, width) * -1, row_height)

            counter += 1

    no_loop()


def mouse_released():
    loop()


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == "c" or key == "C":
        # ------ save an ase file (adobe swatch export) ------
        # create palette
        colors = [None] * color_count
        for i in range(color_count):
            colors[i] = hsva(hue_values[i], saturation_values[i] / 100, brightness_values[i] / 100)
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

# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
