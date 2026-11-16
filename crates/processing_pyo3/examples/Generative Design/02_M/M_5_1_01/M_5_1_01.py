# M_5_1_01
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

# simple example of a recursive function
#
# KEYS
# 1-9                 : recursion level
# s                   : save png
from datetime import datetime

from processing_py import *

recursion_level = 6
start_radius = 200


def setup():
    size(800, 800)


def draw():
    background(1.0)
    no_fill()
    stroke_cap(PROJECT)

    translate(width / 2, height / 2)
    draw_branch(0, 0, start_radius, recursion_level)


# ------ recursive function ------
def draw_branch(x, y, radius, level):
    # draw arc
    stroke_weight(level * 2)
    stroke(0.0, 0.51, 0.64, 100 / 255)  # GAP: translucent colors are blended in linear space, the arcs come out paler and greyer than in Processing
    no_fill()
    arc(x, y, radius * 2, radius * 2, -PI, 0)

    # draw center dot
    fill(0.0)
    no_stroke()
    ellipse(x, y, level * 1.5, level * 1.5)  # GAP: the zero-size ellipse at level 0 logs "slab_allocator: Use-after-free" errors

    # as long as level is greater than zero, draw sub-branches
    if level > 0:
        # left branch
        draw_branch(x - radius, y + radius / 2, radius / 2, level - 1)
        # right branch
        draw_branch(x + radius, y + radius / 2, radius / 2, level - 1)


def key_released():
    global recursion_level
    if key == "s" or key == "S":
        save_frame(timestamp() + "_####.png")

    if key is not None:
        k = ord(key) - 49
        if k >= 0 and k < 9:
            recursion_level = k


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
