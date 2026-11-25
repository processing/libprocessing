# M_2_3_01
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

# draws an amplitude modulated oscillator
#
# KEYS
# i                 : toggle draw info signal
# c                 : toggle draw carrier signal
# 1/2               : info signal frequency -/+
# arrow left/right  : info signal phi -/+
# 7/8               : carrier signal frequency -/+ (modulation frequency)
# s                 : save png
from datetime import datetime

from processing_py import *

point_count = 0
freq = 2
phi = 0
mod_freq = 12

draw_frequency = True
draw_modulation = True
draw_combination = True

angle = 0
y = 0


def setup():
    global point_count
    size(800, 400)
    point_count = width


def draw():
    global angle, y
    background(1.0)
    stroke_weight(1)
    no_fill()

    translate(0, height / 2)

    # draw oscillator with freq and phi
    if draw_frequency:
        stroke(0.0, 0.51, 0.64)
        begin_shape()
        for i in range(point_count + 1):
            angle = remap(i, 0, point_count, 0, TWO_PI)
            y = sin(angle * freq + radians(phi))
            y = y * (height // 4)
            vertex(i, y)
        end_shape()

    # draw oscillator with modFreq
    if draw_modulation:
        stroke(0.0, 0.51, 0.64, 0.5)
        begin_shape()
        for i in range(point_count + 1):
            angle = remap(i, 0, point_count, 0, TWO_PI)
            y = cos(angle * mod_freq)
            y = y * (height // 4)
            vertex(i, y)
        end_shape()

    # draw both combined
    stroke(0.0)
    stroke_weight(2)
    begin_shape()
    for i in range(point_count + 1):
        angle = remap(i, 0, point_count, 0, TWO_PI)
        info = sin(angle * freq + radians(phi))
        carrier = cos(angle * mod_freq)
        y = info * carrier
        y = y * (height // 4)
        vertex(i, y)
    end_shape()


def key_pressed():
    global draw_frequency, draw_modulation, freq, phi, mod_freq
    if key == "s" or key == "S":
        save_frame(timestamp() + ".png")

    if key == "i" or key == "I":
        draw_frequency = not draw_frequency
    if key == "c" or key == "C":
        draw_modulation = not draw_modulation

    if key == "1":
        freq -= 1
    if key == "2":
        freq += 1
    freq = max(freq, 1)

    if key_code == LEFT_ARROW:
        phi -= 15
    if key_code == RIGHT_ARROW:
        phi += 15

    if key == "7":
        mod_freq -= 1
    if key == "8":
        mod_freq += 1
    mod_freq = max(mod_freq, 1)

    print("freq: " + str(freq) + ", phi: " + str(phi) + ", modFreq: " + str(mod_freq))


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
