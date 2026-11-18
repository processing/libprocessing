# P_1_1_1_01
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

# draw the color spectrum by moving the mouse
#
# MOUSE
# position x/y        : resolution
#
# KEYS
# s                   : save png
from datetime import datetime

from processing_py import *

step_x = 0
step_y = 0


def setup():
    size(800, 400)
    background(0.0)


def draw():
    global step_x, step_y
    no_stroke()

    step_x = int(mouse_x) + 2
    step_y = int(mouse_y) + 2
    for grid_y in range(0, height, step_y):
        for grid_x in range(0, width, step_x):
            fill(hsva(grid_x / width * 360, (height - grid_y) / height, 1.0))
            rect(grid_x, grid_y, step_x, step_y)


def key_pressed():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
