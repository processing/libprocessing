# M_2_1_01
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

# draws an oscillator
#
# KEYS
# a                 : toggle oscillation animation
# 1/2               : frequency -/+
# arrow left/right  : phi -/+
# s                 : save png
from datetime import datetime

from processing_py import *

point_count = 0
freq = 1
phi = 0

angle = 0
y = 0

do_draw_animation = True


def setup():
    size(800, 400)


def draw():
    global point_count, angle, y
    background(1.0)
    stroke(0.0)
    stroke_weight(2)
    no_fill()

    if do_draw_animation:
        point_count = width - 250
        translate(250, height / 2)
    else:
        point_count = width
        translate(0, height / 2)

    # draw oscillator curve
    begin_shape()
    for i in range(point_count + 1):
        angle = remap(i, 0, point_count, 0, TWO_PI)
        y = sin(angle * freq + radians(phi))
        y = y * 100
        vertex(i, y)
    end_shape()

    if do_draw_animation:
        draw_animation()


def draw_animation():
    global angle, y
    # draw circle, dots and lines
    t = (frame_count / point_count) % 1
    angle = remap(t, 0, 1, 0, TWO_PI)
    x = cos(angle * freq + radians(phi))
    x = x * 100 - 125
    y = sin(angle * freq + radians(phi))
    y = y * 100

    # circle
    stroke_weight(1)
    ellipse(-125, 0, 200, 200)

    # lines
    stroke(0.0, 0.5)
    line(0, -100, 0, 100)
    line(0, 0, point_count, 0)
    line(-225, 0, -25, 0)
    line(-125, -100, -125, 100)
    line(x, y, -125, 0)

    stroke(0.0, 0.51, 0.64)
    stroke_weight(2)
    line(t * point_count, y, t * point_count, 0)
    line(x, y, x, 0)

    phi_x = cos(radians(phi)) * 100 - 125
    phi_y = sin(radians(phi)) * 100

    # phi line
    stroke_weight(1)
    stroke(0.0, 0.5)
    line(-125, 0, phi_x, phi_y)

    # phi dots
    fill(0.0)
    stroke(1.0)
    stroke_weight(2)
    ellipse(0, phi_y, 8, 8)
    ellipse(phi_x, phi_y, 8, 8)

    # dot on curve
    ellipse(t * point_count, y, 10, 10)

    # dot on circle
    ellipse(x, y, 10, 10)


def key_pressed():
    global do_draw_animation, freq, phi
    if key == "s" or key == "S":
        save_frame(timestamp() + ".png")

    if key == "a" or key == "A":
        do_draw_animation = not do_draw_animation

    if key == "1":
        freq -= 1
    if key == "2":
        freq += 1
    freq = max(freq, 1)

    if key_code == LEFT_ARROW:
        phi -= 15
    if key_code == RIGHT_ARROW:
        phi += 15

    print("freq: " + str(freq) + ", phi: " + str(phi))


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
