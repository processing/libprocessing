# M_2_2_01
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

# draws a lissajous curve
#
# KEYS
# a                 : toggle oscillation animation
# 1/2               : frequency x -/+
# 3/4               : frequency y -/+
# arrow left/right  : phi -/+
# s                 : save png
from datetime import datetime

from processing_py import *

point_count = 600
freq_x = 1
freq_y = 2
phi = 90

angle = 0
x = 0
y = 0
factor_x = 0
factor_y = 0

do_draw_animation = True

margin = 25


def setup():
    size(600, 600)


def draw():
    global angle, x, y, factor_x, factor_y
    background(1.0)
    stroke(0.0)
    stroke_weight(1)

    if do_draw_animation:
        translate(width * 3 / 4.0, height * 3 / 4.0)
        factor_x = width // 4 - margin
        factor_y = height // 4 - margin
    else:
        translate(width / 2, height / 2)
        factor_x = width // 2 - margin
        factor_y = height // 2 - margin

    begin_shape()
    for i in range(point_count + 1):
        angle = remap(i, 0, point_count, 0, TWO_PI)

        x = sin(angle * freq_x + radians(phi))
        y = sin(angle * freq_y)

        x = x * factor_x
        y = y * factor_y

        vertex(x, y)
    end_shape()

    if do_draw_animation:
        draw_animation()


def draw_animation():
    global angle, x, y
    push_style()
    no_fill()

    # draw x oscillator
    stroke(0.0)
    begin_shape()
    for i in range(point_count + 1):
        angle = remap(i, 0, point_count, 0, TWO_PI)
        x = sin(angle * freq_x + radians(phi))
        x = x * (width // 4 - margin)
        y = -height * 2 / 3.0 - margin + i / point_count * (height // 2 - 2 * margin)
        vertex(x, y)
    end_shape()

    # draw y oscillator
    stroke(0.0)
    begin_shape()
    for i in range(point_count + 1):
        angle = remap(i, 0, point_count, 0, TWO_PI)
        y = sin(angle * freq_y)
        y = y * (height // 4 - margin)
        x = -width * 2 / 3.0 - margin + i / point_count * (width // 2 - 2 * margin)
        vertex(x, y)
    end_shape()

    angle = remap(frame_count, 0, point_count, 0, TWO_PI)
    x = sin(angle * freq_x + radians(phi))
    y = sin(angle * freq_y)

    x = x * (width // 4 - margin)
    y = y * (height // 4 - margin)

    osc_yx = -width * 2 / 3.0 - margin + (angle / TWO_PI) % 1 * (width // 2 - 2 * margin)
    osc_xy = -height * 2 / 3.0 - margin + (angle / TWO_PI) % 1 * (height // 2 - 2 * margin)

    stroke(0.0, 80 / 255)
    line(x, osc_xy, x, y)
    line(osc_yx, y, x, y)

    fill(0.0)
    stroke(1.0)
    stroke_weight(2)

    ellipse(x, osc_xy, 8, 8)
    ellipse(osc_yx, y, 8, 8)

    ellipse(x, y, 10, 10)

    pop_style()


def key_pressed():
    global do_draw_animation, freq_x, freq_y, phi
    if key == "s" or key == "S":
        save_frame(timestamp() + ".png")

    if key == "a" or key == "A":
        do_draw_animation = not do_draw_animation

    if key == "1":
        freq_x -= 1
    if key == "2":
        freq_x += 1
    freq_x = max(freq_x, 1)

    if key == "3":
        freq_y -= 1
    if key == "4":
        freq_y += 1
    freq_y = max(freq_y, 1)

    if key_code == LEFT_ARROW:
        phi -= 15
    if key_code == RIGHT_ARROW:
        phi += 15

    print("freqX: " + str(freq_x) + ", freqY: " + str(freq_y) + ", phi: " + str(phi))


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
