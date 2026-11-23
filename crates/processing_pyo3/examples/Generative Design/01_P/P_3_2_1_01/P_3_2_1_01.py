# P_3_2_1_01
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

# typo outline displayed as dots and lines
#
# KEYS
# a-z                  : text input (keyboard)
# backspace            : delete last typed letter
# ctrl                 : save png
from datetime import datetime

from processing_py import *

font = None
text_typed = "Type ...!"

do_save = False

# distance between two points on the outline
segment_length = 11


def setup():
    global font
    size(1324, 350)
    # make window resizable
    window_resizable(True)

    # load the font in setup
    font = load_font("data/FreeSans.ttf")
    text_font(font)
    text_size(200)


def draw():
    global do_save
    background(1.0)
    # margin border
    translate(20, 220)

    if len(text_typed) > 0:
        # get the points on font outline
        pnts = text_to_points(text_typed, 0, 0, 1 / segment_length)

        # lines
        stroke(0.71, 0.62, 0.0)
        stroke_weight(1.0)
        for i in range(len(pnts)):
            l = 5
            line(pnts[i][0] - l, pnts[i][1] - l, pnts[i][0] + l, pnts[i][1] + l)

        # dots
        fill(0.0)
        no_stroke()
        for i in range(len(pnts)):
            diameter = 7
            # on ervery second point
            if i % 2 == 0:
                ellipse(pnts[i][0], pnts[i][1], diameter, diameter)

        if do_save:
            do_save = False
            save_frame(timestamp() + "_####.png")


def key_pressed():
    global text_typed, do_save
    if key_code == DELETE or key_code == BACKSPACE:
        text_typed = text_typed[0 : max(0, len(text_typed) - 1)]
    elif key_code == TAB or key_code == ENTER or key_code == ESCAPE:
        pass
    elif key is not None:
        text_typed += key

    if key_code == CONTROL:
        do_save = True


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
