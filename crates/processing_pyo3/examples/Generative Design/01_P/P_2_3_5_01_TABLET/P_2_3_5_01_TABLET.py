# P_2_3_5_01_TABLET
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

# draw tool. shows how to draw with dynamic elements.
# works only with an external tablet device!
#
# The generativedesign Tablet of the original is replaced by a small class
# that reads the mouse: constant pressure while a button is down, and a
# constant azimuth and altitude.
#
# MOUSE
# drag                : draw
#
# TABLET
# pressure            : saturation (in draw mode 3 only)
# azimuth             : rotation of each element
# altitude            : length of each element
#
# KEYS
# 1-3                 : draw mode
# 6-0                 : colors
# del, backspace      : clear screen
# s                   : save png
from datetime import datetime

from processing_py import *


class Tablet:
    # the mouse has no pressure, azimuth or altitude, so constant values are reported
    def get_pressure(self):
        if mouse_is_pressed:
            return 0.5
        return 0.0

    def get_azimuth(self):
        return QUARTER_PI

    def get_altitude(self):
        return QUARTER_PI


tablet = None

draw_mode = 1

back = color(1.0)
from_color = color(181 / 255, 157 / 255, 0.0)
to_color = color(181 / 255, 157 / 255, 0.0)


def setup():
    global tablet
    size(1280, 960)
    tablet = Tablet()
    background(back)
    frame_rate(30)

    # # load an image in background
    # img = load_image(select_input("select a background image"))
    # image(img, 0, 0, width, height)


def draw():
    # gamma values optimized for wacom intuos 3
    pressure = gamma(tablet.get_pressure(), 2.5)
    angle = tablet.get_azimuth()
    pen_length = cos(tablet.get_altitude())

    if pressure > 0.0 and pen_length > 0.0:
        push_matrix()
        translate(mouse_x, mouse_y)
        rotate(angle)

        element_length = pen_length * 250
        h1 = random(10) * (1.2 + pen_length)
        h2 = (-10 + random(10)) * (1.2 + pen_length)

        if draw_mode == 1:
            stroke(1.0)
            stroke_weight(0.5)
            fill(0.0)
        elif draw_mode == 2:
            stroke(1.0)
            stroke_weight(0.5)
            # no_stroke()
            white = color(1.0, 20 / 255)
            inter_color = lerp_color(from_color, to_color, random(0, 1))
            fill(lerp_color(white, inter_color, pressure))
        elif draw_mode == 3:
            julia = remap(pressure, 0, 1, 0.0, 1.0)
            fill(0.0, julia)
            stroke(1.0)
            stroke_weight(0.5)
            element_length = pen_length * 50
            h1 = random(5) + element_length / 2
            h2 = (random(5) + element_length / 2) * -1

        points_x = [0.0] * 5
        points_y = [0.0] * 5

        points_x[0] = 0
        points_y[0] = 0

        points_x[1] = element_length * 0.77
        points_y[1] = h1

        points_x[2] = element_length
        points_y[2] = 0

        points_x[3] = element_length * 0.77
        points_y[3] = h2

        points_x[4] = 0
        points_y[4] = -5

        begin_shape()
        # start controlpoint
        curve_vertex(points_x[3], points_y[3])

        # only these points are drawn
        for i in range(len(points_x)):
            curve_vertex(points_x[i], points_y[i])

        # end controlpoint
        curve_vertex(points_x[1], points_y[1])
        end_shape(CLOSE)

        pop_matrix()


def key_released():
    global draw_mode, from_color, to_color
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key_code == DELETE or key_code == BACKSPACE:
        background(back)

    # switch drawing styles
    if key == "1":
        draw_mode = 1
    if key == "2":
        draw_mode = 2
    if key == "3":
        draw_mode = 3

    if key == "6":
        from_color = color(181 / 255, 157 / 255, 0.0)
        to_color = color(181 / 255, 157 / 255, 0.0)
    if key == "7":
        from_color = color(0.0, 130 / 255, 164 / 255, 20 / 255)
        to_color = color(0.0, 130 / 255, 164 / 255, 20 / 255)
    if key == "8":
        from_color = color(87 / 255, 35 / 255, 129 / 255)
        to_color = color(87 / 255, 35 / 255, 129 / 255)
    if key == "9":
        from_color = color(197 / 255, 0.0, 123 / 255)
        to_color = color(197 / 255, 0.0, 123 / 255)
    if key == "0":
        from_color = color(0.0, 130 / 255, 164 / 255)
        to_color = color(87 / 255, 35 / 255, 129 / 255)


# gamma ramp, non linaer mapping ...
def gamma(the_value, the_gamma):
    return pow(the_value, the_gamma)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
