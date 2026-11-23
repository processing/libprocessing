# P_3_2_3_01
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

# fontgenerator with dynamic elements. letter ouline consist of linked agents.
#
# MOUSE
# press + position x  : letter distortion
#
# KEYS
# a-z                 : text input (keyboard)
# shift               : freeze current state
# del, backspace      : clear screen
# tab                 : save png
from datetime import datetime

from processing_py import *

typed_key = "a"
spacing = 20
space_width = 150  # width of letter ' '
font_size = 200
line_spacing = font_size * 1.5
step_size = 2
dance_factor = 1
letter_x = 50
text_w = 50
letter_y = line_spacing

# distance between two points on the outline
segment_length = 25

font = None
pnts = []

freeze = False


def setup():
    global font, pnts, text_w
    size(1200, 800)
    # make window resizable
    window_resizable(True)

    frame_rate(15)

    font = load_font("data/FreeSansNoPunch.ttf")
    text_font(font)
    text_size(font_size)

    #  ------ get the points on the curve's shape  ------
    pnts = text_to_points(typed_key, 0, 0, 1 / segment_length)
    text_w = get_width(pnts)

    background(1.0)


def draw():
    global dance_factor
    no_fill()
    push_matrix()

    # translation according the amoutn of letters
    translate(letter_x, letter_y)

    # distortion on/off
    if mouse_is_pressed:
        dance_factor = remap(mouse_x, 0, width, 1, 3)
    else:
        dance_factor = 1

    # are there points to draw?
    if len(pnts) > 0:
        # let the points dance
        for i in range(len(pnts)):
            pnts[i][0] += random(-step_size, step_size) * dance_factor
            pnts[i][1] += random(-step_size, step_size) * dance_factor

        #  ------ lines: connected rounded  ------
        stroke_weight(0.08)
        # stroke(0.78, 0.0, 0.0)
        begin_shape()
        # start controlpoint
        curve_vertex(pnts[len(pnts) - 1][0], pnts[len(pnts) - 1][1])
        # only these points are drawn
        for i in range(len(pnts)):
            curve_vertex(pnts[i][0], pnts[i][1])
        curve_vertex(pnts[0][0], pnts[0][1])
        # end controlpoint
        curve_vertex(pnts[1][0], pnts[1][1])
        end_shape()

        #  ------ lines: connected straight  ------
        stroke_weight(0.1)
        stroke(0.0)
        begin_shape()
        for i in range(len(pnts)):
            vertex(pnts[i][0], pnts[i][1])
            ellipse(pnts[i][0], pnts[i][1], 7, 7)
        vertex(pnts[0][0], pnts[0][1])
        end_shape()

    pop_matrix()


# width of the letter outline (the distance from its leftmost to its rightmost point)
def get_width(points):
    if len(points) == 0:
        return 0
    min_x = points[0][0]
    max_x = points[0][0]
    for i in range(len(points)):
        min_x = min(min_x, points[i][0])
        max_x = max(max_x, points[i][0])
    return max_x - min_x


def key_released():
    global freeze
    if key_code == TAB:
        save_frame(timestamp() + "_##.png")
    if key_code == SHIFT:
        # switch loop on/off
        freeze = not freeze
        if freeze == True:
            no_loop()
        else:
            loop()


def key_pressed():
    global typed_key, pnts, text_w, letter_x, letter_y, freeze
    if key_code == ENTER:
        pnts = []
        letter_y += line_spacing
        text_w = letter_x = 20
    elif key_code == ESCAPE or key_code == TAB:
        pass
    elif key_code == BACKSPACE or key_code == DELETE:
        background(1.0)
        pnts = []
        text_w = letter_x = 0
        letter_y = line_spacing
        freeze = False
        loop()
    elif key == " ":
        pnts = []
        letter_x += space_width
        freeze = False
        loop()
    elif key is not None:
        typed_key = key
        # add to actual pos the letter width
        text_w += spacing
        letter_x += text_w
        pnts = text_to_points(typed_key, 0, 0, 1 / segment_length)
        text_w = get_width(pnts)
        freeze = False
        loop()


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
