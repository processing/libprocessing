# P_4_3_3_02
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

# generating a drawing with 3 brushes by analysing live video
#
# KEYS
# q                   : stop drawing
# w                   : continue drawing
# s                   : save png
from datetime import datetime

from processing_py import *

video = None

pixel_index = 0
c = None

x1 = 0
x2 = 0
x3 = 0
y1 = 0
y2 = 0
y3 = 0
curve_point_x = 0
curve_point_y = 0

counter = 0
max_counter = 100000


def setup():
    global video, x1, y1, x2, y2, x3, y3
    size(640, 480)
    video = create_webcam(width, height, 30)  # GAP: create_webcam() is missing from the shipped build (the engine's "webcam" feature is off)
    background(1.0)
    x1 = 0
    y1 = height // 2
    x2 = width // 2
    y2 = 0
    x3 = width
    y3 = height // 2


def draw():
    global pixel_index, c, x1, y1, x2, y2, x3, y3, curve_point_x, curve_point_y, counter
    no_fill()

    # get actual web cam image
    video.load_pixels()  # GAP: Webcam.load_pixels() is missing

    # first line
    pixel_index = int((video.width - 1 - int(x1)) + int(y1) * video.width)  # GAP: Webcam.width is missing
    c = video.pixels[pixel_index]  # GAP: Webcam.pixels is missing
    hue_value = hue(c)
    stroke_weight(hue_value / 50)
    stroke(c)

    begin_shape()
    curve_vertex(x1, y1)
    curve_vertex(x1, y1)
    for i in range(7):
        curve_point_x = constrain(x1 + random(-50, 50), 0, width - 1)
        curve_point_y = constrain(y1 + random(-50, 50), 0, height - 1)
        curve_vertex(curve_point_x, curve_point_y)
    curve_vertex(curve_point_x, curve_point_y)
    end_shape()
    x1 = curve_point_x
    y1 = curve_point_y

    # second line
    pixel_index = int((video.width - 1 - int(x2)) + int(y2) * video.width)
    c = video.pixels[pixel_index]
    saturation_value = saturation(c) * 100
    stroke_weight(saturation_value / 2)
    stroke(c)

    begin_shape()
    curve_vertex(x2, y2)
    curve_vertex(x2, y2)
    for i in range(7):
        curve_point_x = constrain(x2 + random(-50, 50), 0, width - 1)
        curve_point_y = constrain(y2 + random(-50, 50), 0, height - 1)
        curve_vertex(curve_point_x, curve_point_y)
    curve_vertex(curve_point_x, curve_point_y)
    end_shape()
    x2 = curve_point_x
    y2 = curve_point_y

    # third line
    pixel_index = int((video.width - 1 - int(x3)) + int(y3) * video.width)
    c = video.pixels[pixel_index]
    brightness_value = brightness(c) * 100
    stroke_weight(brightness_value / 100)
    stroke(c)

    begin_shape()
    curve_vertex(x3, y3)
    curve_vertex(x3, y3)
    for i in range(7):
        curve_point_x = constrain(x3 + random(-50, 50), 0, width - 1)
        curve_point_y = constrain(y3 + random(-50, 50), 0, height - 1)
        curve_vertex(curve_point_x, curve_point_y)
    curve_vertex(curve_point_x, curve_point_y)
    end_shape()
    x3 = curve_point_x
    y3 = curve_point_y

    counter += 1
    if counter >= max_counter:
        no_loop()


def key_pressed():
    if key_code == BACKSPACE:
        background(1.0)


def key_released():
    if key == "s":
        save_frame(timestamp() + "_##.png")
    if key == "q":
        no_loop()
    if key == "w":
        loop()


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
