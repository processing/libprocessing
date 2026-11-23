# P_4_3_3_01
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

# generating a drawing by analysing live video
#
# MOUSE
# position x          : drawing speed
# position y          : diffusion
#
# KEYS
# arrow up            : number of curve points +
# arrow down          : number of curve points -
# q                   : stop drawing
# w                   : continue drawing
# s                   : save png
from datetime import datetime

from processing_py import *

video = None

x = 0
y = 0
curve_point_x = 0
curve_point_y = 0
point_count = 1
diffusion = 50


def setup():
    global x, y, video
    size(640, 480)
    background(1.0)
    x = width // 2
    y = height // 2
    video = create_webcam(width, height, 30)  # GAP: create_webcam() is missing from the shipped build (the engine's "webcam" feature is off)


def draw():
    global x, y, curve_point_x, curve_point_y, diffusion
    no_fill()

    for j in range(mouse_x // 50 + 1):
        # get actual web cam image
        video.load_pixels()  # GAP: Webcam.load_pixels() is missing

        # first line
        pixel_index = (video.width - 1 - x) + y * video.width  # GAP: Webcam.width is missing
        c = video.pixels[pixel_index]  # GAP: Webcam.pixels is missing
        # hue_value = hue(c)
        stroke_weight(hue(c) / 50)
        stroke(c)

        diffusion = remap(mouse_y, 0, height, 5, 100)

        begin_shape()
        curve_vertex(x, y)
        curve_vertex(x, y)
        for i in range(point_count):
            rx = int(random(-diffusion, diffusion))
            curve_point_x = constrain(x + rx, 0, width - 1)
            ry = int(random(-diffusion, diffusion))
            curve_point_y = constrain(y + ry, 0, height - 1)
            curve_vertex(curve_point_x, curve_point_y)
        curve_vertex(curve_point_x, curve_point_y)
        end_shape()

        x = curve_point_x
        y = curve_point_y


def key_pressed():
    if key_code == BACKSPACE:
        background(1.0)


def key_released():
    global point_count
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "q" or key == "S":
        no_loop()
    if key == "w" or key == "W":
        loop()

    if key_code == UP:
        point_count = min(point_count + 1, 30)
    if key_code == DOWN:
        point_count = max(point_count - 1, 1)


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
