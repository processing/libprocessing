# P_1_1_2_01
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

# changing the color circle by moving the mouse.
#
# MOUSE
# position x          : saturation
# position y          : brighness
#
# KEYS
# 1-5                 : number of segments
# s                   : save png
from datetime import datetime

from processing_py import *

segment_count = 360
radius = 300


def setup():
    size(800, 800)


def draw():
    no_stroke()
    background(1.0)

    angle_step = 360 // segment_count

    begin_shape(TRIANGLE_FAN)
    vertex(width / 2, height / 2)
    for angle in range(0, 361, angle_step):
        vx = width / 2 + cos(radians(angle)) * radius
        vy = height / 2 + sin(radians(angle)) * radius
        vertex(vx, vy)
        fill(hsva(angle, mouse_x / width, mouse_y / height))  # GAP: per-vertex fill() inside begin_shape() is ignored, the whole shape takes one color
    end_shape()


def key_released():
    global segment_count
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        segment_count = 360
    elif key == "2":
        segment_count = 45
    elif key == "3":
        segment_count = 24
    elif key == "4":
        segment_count = 12
    elif key == "5":
        segment_count = 6


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
