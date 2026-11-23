# P_2_2_3_01
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

# form mophing process by connected random agents
#
# MOUSE
# click               : start a new circe
# position x/y        : direction of floating
#
# KEYS
# 1-2                 : fill styles
# f                   : freeze. loop on/off
# Delete/Backspace    : clear display
# s                   : save png
from datetime import datetime

from processing_py import *

form_resolution = 15
step_size = 2
distortion_factor = 1
init_radius = 150
center_x = 0
center_y = 0
x = [0.0] * form_resolution
y = [0.0] * form_resolution

filled = False
freeze = False


def setup():
    global center_x, center_y
    size(display_width, display_height)  # GAP: display_width / display_height are 0 inside setup(), so the window cannot be created

    # init form
    center_x = width / 2
    center_y = height / 2
    angle = radians(360 / float(form_resolution))
    for i in range(form_resolution):
        x[i] = cos(angle * i) * init_radius
        y[i] = sin(angle * i) * init_radius

    stroke(0.0, 0.2)
    background(1.0)


def draw():
    global center_x, center_y
    # floating towards mouse position
    if mouse_x != 0 or mouse_y != 0:
        center_x += (mouse_x - center_x) * 0.01
        center_y += (mouse_y - center_y) * 0.01

    # calculate new points
    for i in range(form_resolution):
        x[i] += random(-step_size, step_size)
        y[i] += random(-step_size, step_size)
        # ellipse(x[i], y[i], 5, 5)

    stroke_weight(0.75)
    if filled:
        fill(random(1.0))
    else:
        no_fill()

    begin_shape()
    # start controlpoint
    curve_vertex(x[form_resolution - 1] + center_x, y[form_resolution - 1] + center_y)

    # only these points are drawn
    for i in range(form_resolution):
        curve_vertex(x[i] + center_x, y[i] + center_y)
    curve_vertex(x[0] + center_x, y[0] + center_y)

    # end controlpoint
    curve_vertex(x[1] + center_x, y[1] + center_y)
    end_shape()


# events
def mouse_pressed():
    global center_x, center_y
    # init form on mouse position
    center_x = mouse_x
    center_y = mouse_y
    angle = radians(360 / float(form_resolution))
    radius = init_radius * random(0.5, 1.0)
    for i in range(form_resolution):
        x[i] = cos(angle * i) * radius
        y[i] = sin(angle * i) * radius


def key_released():
    global filled, freeze
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)

    if key == "1":
        filled = False
    if key == "2":
        filled = True

    # switch draw loop on/off
    if key == "f" or key == "F":
        freeze = not freeze
    if freeze == True:
        no_loop()
    else:
        loop()


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
