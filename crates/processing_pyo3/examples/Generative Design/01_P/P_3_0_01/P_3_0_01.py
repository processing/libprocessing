# P_3_0_01
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

# changing the size and the position of a letter
#
# MOUSE
# position x          : size
# position y          : position
# drag                : draw
#
# KEYS
# a-z                 : change letter
# ctrl                : save png
from datetime import datetime

from processing_py import *

font = None
letter = "A"


def setup():
    global font
    size(800, 800)
    background(1.0)
    fill(0.0)

    font = create_font("Arial")
    text_font(font)
    text_size(12)
    text_align(CENTER, CENTER)


def draw():
    pass


def mouse_moved():
    background(1.0)
    text_size((mouse_x - width // 2) * 5 + 1)
    text(letter, width / 2, mouse_y)


def mouse_dragged():
    text_size((mouse_x - width // 2) * 5 + 1)
    text(letter, width / 2, mouse_y)


def key_released():
    global letter
    if key_code == CONTROL:
        save_frame(timestamp() + "_##.png")

    if key is not None and ord(key) > 32:
        letter = key


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
