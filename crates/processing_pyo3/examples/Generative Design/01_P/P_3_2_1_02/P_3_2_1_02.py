# P_3_2_1_02
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

# CREDITS
# Geomerative library by Ricard Marxer (the original; this port uses text_to_points).
# FreeSans.ttf (GNU FreeFont), see the readme files in data folder.

# fontgenerator with static elements (svg files)
#
# MOUSE
# position x          : module rotation
# position y          : module size
#
# KEYS
# a-z                 : text input (keyboard)
# alt                 : toggle modules
# del, backspace      : remove last letter
# ctrl                : save png
from datetime import datetime

from processing_py import *

do_save = False

font = None

text_typed = "Type ...!"

shape_set = 0
module1 = None
module2 = None

# distance between two points on the outline
segment_length = 6


def setup():
    global font, module1, module2
    size(1024, 500)
    # make window resizable
    window_resizable(True)

    module1 = load_shape("data/A_01.svg")  # GAP: load_shape() (SVG) is missing
    module2 = load_shape("data/A_02.svg")  # GAP: load_shape() (SVG) is missing
    module1.disable_style()  # GAP: Shape.disable_style(); ignore the colors in the SVG
    module2.disable_style()  # GAP: Shape.disable_style()

    # load a truetype font
    font = load_font("data/FreeSans.ttf")
    text_font(font)
    text_size(200)


def draw():
    global do_save
    background(1.0)
    no_stroke()
    shape_mode(CENTER)  # GAP: shape_mode() is missing

    # margin border
    translate(60, 300)

    if len(text_typed) > 0:
        # get the points on font outline
        pnts = text_to_points(text_typed, 0, 0, 1 / segment_length)

        # ------ svg modules ------
        # module1
        fill(0.71, 0.62, 0.0, 0.78)
        diameter = 30
        for i in range(len(pnts) - 1):
            # on every third point
            if i % 3 == 0:
                # rotate the module facing to the next one (i+1)
                push_matrix()
                angle = atan2(pnts[i][1] - pnts[i + 1][1], pnts[i][0] - pnts[i + 1][0])
                translate(pnts[i][0], pnts[i][1])
                rotate(angle)
                rotate(radians(-mouse_x))
                shape(module1, 0, 0, diameter + (mouse_y / 2.5), diameter + (mouse_y / 2.5))  # GAP: shape(s, x, y, w, h)
                pop_matrix()

        # module2
        fill(0.0, 0.51, 0.64, 0.78)
        diameter = 18
        for i in range(len(pnts) - 1):
            # on every third point
            if i % 3 == 0:
                # rotate the module facing to the next one (i+1)
                push_matrix()
                angle = atan2(pnts[i][1] - pnts[i + 1][1], pnts[i][0] - pnts[i + 1][0])
                translate(pnts[i][0], pnts[i][1])
                rotate(angle)
                rotate(radians(mouse_x))
                shape(module2, 0, 0, diameter + (mouse_y / 2.5), diameter + (mouse_y / 2.5))  # GAP: shape(s, x, y, w, h)
                pop_matrix()

        if do_save:
            do_save = False
            save_frame(timestamp() + ".png")


def key_released():
    global do_save
    if key_code == CONTROL:
        do_save = True


def key_pressed():
    global text_typed, shape_set, module1, module2
    if key == "1" or key_code == DELETE or key_code == BACKSPACE:
        text_typed = text_typed[0 : max(0, len(text_typed) - 1)]
    elif key_code == TAB or key_code == ENTER or key_code == ESCAPE:
        pass
    elif key is not None:
        text_typed += key

    if key_code == ALT:
        shape_set = (shape_set + 1) % 4
        if shape_set == 0:
            module1 = load_shape("data/A_01.svg")  # GAP: load_shape() (SVG) is missing
            module2 = load_shape("data/A_02.svg")  # GAP: load_shape() (SVG) is missing
        elif shape_set == 1:
            module1 = load_shape("data/B_01.svg")  # GAP: load_shape() (SVG) is missing
            module2 = load_shape("data/B_02.svg")  # GAP: load_shape() (SVG) is missing
        elif shape_set == 2:
            module1 = load_shape("data/C_01.svg")  # GAP: load_shape() (SVG) is missing
            module2 = load_shape("data/C_02.svg")  # GAP: load_shape() (SVG) is missing
        elif shape_set == 3:
            module1 = load_shape("data/D_01.svg")  # GAP: load_shape() (SVG) is missing
            module2 = load_shape("data/D_02.svg")  # GAP: load_shape() (SVG) is missing
        module1.disable_style()  # GAP: Shape.disable_style()
        module2.disable_style()  # GAP: Shape.disable_style()


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
