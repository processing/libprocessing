# P_4_1_2_01
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

# image feedback process.
#
# KEYS
# del, backspace      : clear screen
# s                   : save png
from datetime import datetime

from processing_py import *

img = None


def setup():
    global img
    size(1024, 780)
    background(1.0)
    img = load_image("data/pic.png")
    image(img, 0, 100)


def draw():
    x1 = int(random(0, width))
    y1 = 0

    x2 = round(x1 + random(-7, 7))
    y2 = round(random(-5, 5))

    w = int(random(35, 50))
    h = height

    copy(x1, y1, w, h, x2, y2, w, h)  # GAP: copy(sx, sy, sw, sh, dx, dy, dw, dh) of a canvas region onto the canvas itself is missing; copy(src, src_rect=, dst_rect=) needs an Image or offscreen Graphics as src


def key_released():
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)
        image(img, 0, 100)
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
