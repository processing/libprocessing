# P_3_2_2_01
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

# fontgenerator with dynamic elements
#
# MOUSE
# position x          : curve rotation
# position y          : curve height
#
# KEYS
# a-z                 : text input (keyboard)
# del, backspace      : remove last letter
# alt                 : toggle fill style
# ctrl                : save png
from datetime import datetime

from processing_py import *

do_save = False

font = None
# text_typed = "HELLO"

# text_typed = "When you’re creating"
# text_typed = "your own shit, man,"
# text_typed = "even the sky ain’t"
# text_typed = "  the limit."
text_typed = "Charles Mingus"

# text_typed = "WHEN YOU’RE CREATING"
# text_typed = "your own shit, man,"
# text_typed = "even the sky ain’t"
# text_typed = "the limit."


filled = False

# distance between two points on the outline
segment_length = 10


def setup():
    global font
    size(1824, 400)
    # make window resizable
    window_resizable(True)

    # load a truetype font
    font = load_font("data/FreeSans.ttf")
    text_font(font)
    text_size(200)


def draw():
    global do_save
    background(1.0)
    if filled:
        no_stroke()
        fill(0.0)
    else:
        no_fill()
        stroke(0.0)
        stroke_weight(2)

    # initial position
    translate(20, 260)

    if len(text_typed) > 0:
        # get the points on font outline
        pnts = text_to_points(text_typed, 0, 0, 1 / segment_length)

        # map mouse axis
        add_to_angle = remap(mouse_x, 0, width, -PI, +PI)
        curve_height = remap(mouse_y, 0, height, 0.1, 2)

        for i in range(len(pnts) - 1):
            d = dist(pnts[i][0], pnts[i][1], pnts[i + 1][0], pnts[i + 1][1])
            # create a gap between each letter
            if d > 20:
                continue
            # alternate in every step from -1 to 1
            stepper = remap(i % 2, 0, 1, -1, 1)
            angle = atan2(pnts[i + 1][1] - pnts[i][1], pnts[i + 1][0] - pnts[i][0])
            angle = angle + add_to_angle

            cx = pnts[i][0] + cos(angle * stepper) * d * 4 * curve_height
            cy = pnts[i][1] + sin(angle * stepper) * d * 3 * curve_height

            bezier(pnts[i][0], pnts[i][1], cx, cy, cx, cy, pnts[i + 1][0], pnts[i + 1][1])

    if do_save:
        do_save = False
        save_frame(timestamp() + ".png")


def key_released():
    global do_save, filled
    if key_code == CONTROL:
        do_save = True
    if key_code == ALT:
        filled = not filled


def key_pressed():
    global text_typed
    if key_code == DELETE or key_code == BACKSPACE:
        text_typed = text_typed[0 : max(0, len(text_typed) - 1)]
    elif key_code == TAB or key_code == ENTER or key_code == ESCAPE:
        pass
    elif key is not None:
        text_typed += key


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
