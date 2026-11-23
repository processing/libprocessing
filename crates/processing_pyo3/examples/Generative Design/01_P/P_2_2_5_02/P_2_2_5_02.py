# P_2_2_5_02
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

# pack as many cirlces as possible together
#
# MOUSE
# press + position x/y : move area of interest
#
# KEYS
# 1                    : show/hide svg modules
# 2                    : show/hide lines
# 3                    : show/hide circles
# arrow up/down        : resize area of interest
# f                    : freeze process. on/off
# s                    : save png
from datetime import datetime

from processing_py import *

freeze = False

max_count = 5000  # max count of the cirlces
current_count = 1
x = [0.0] * max_count
y = [0.0] * max_count
r = [0.0] * max_count  # radius
closest_index = [0] * max_count  # index

min_radius = 3
max_radius = 50

# mouse and arrow up/down interaction
mouse_rect = 30

# svg vector import
module1 = None
module2 = None

# style selector, hotkeys 1,2,3
show_svg = True
show_line = False
show_circle = False


def setup():
    global module1, module2
    size(800, 800)
    no_fill()
    cursor(CROSS)

    module1 = load_shape("data/01.svg")  # GAP: load_shape() (SVG) is missing
    module2 = load_shape("data/02.svg")  # GAP: load_shape() (SVG) is missing

    # first circle
    x[0] = 200
    y[0] = 100
    r[0] = 50
    closest_index[0] = 0


def draw():
    global current_count
    shape_mode(CENTER)  # GAP: shape_mode() is missing
    ellipse_mode(CENTER)
    background(1.0)

    for j in range(40):
        # create a random position
        tx = random(0 + max_radius, width - max_radius)
        ty = random(0 + max_radius, height - max_radius)
        tr = min_radius

        # create a random position according to mouse position
        if mouse_is_pressed == True:
            tx = random(mouse_x - mouse_rect / 2, mouse_x + mouse_rect / 2)
            ty = random(mouse_y - mouse_rect / 2, mouse_y + mouse_rect / 2)
            tr = 1

        insection = True
        # find a pos with no intersection with others circles
        for i in range(current_count):
            d = dist(tx, ty, x[i], y[i])
            # print(d)
            if d >= (tr + r[i]):
                insection = False
            else:
                insection = True
                break

        # stop process by pressing hotkey 'F'
        if freeze:
            insection = True

        # no intection ... add a new circle
        if insection == False:
            # get closest neighbour and closest possible radius
            t_radius = width
            for i in range(current_count):
                d = dist(tx, ty, x[i], y[i])
                if t_radius > d - r[i]:
                    t_radius = d - r[i]
                    closest_index[current_count] = i

            if t_radius > max_radius:
                t_radius = max_radius

            x[current_count] = tx
            y[current_count] = ty
            r[current_count] = t_radius
            current_count += 1

    # draw them
    for i in range(current_count):
        push_matrix()
        translate(x[i], y[i])
        # we abuse radius as random angle :)
        rotate(radians(r[i]))

        if show_svg:
            # draw SVGs
            if r[i] == max_radius:
                shape(module1, 0, 0, r[i] * 2, r[i] * 2)  # GAP: shape(s, x, y, w, h)
            else:
                shape(module2, 0, 0, r[i] * 2, r[i] * 2)  # GAP: shape(s, x, y, w, h)
        if show_circle:
            # draw circles
            stroke(0.0)
            stroke_weight(1.5)
            ellipse(0, 0, r[i] * 2, r[i] * 2)
        pop_matrix()

        if show_line:
            # draw connection-lines to the nearest neighbour
            stroke(150 / 255)
            stroke_weight(1)
            n = closest_index[i]
            line(x[i], y[i], x[n], y[n])

    # visualize the random range of the new positions
    if mouse_is_pressed == True:
        stroke(1.0, 200 / 255, 0.0)
        stroke_weight(2)
        rect(mouse_x - mouse_rect / 2, mouse_y - mouse_rect / 2, mouse_rect, mouse_rect)

    if current_count >= max_count:
        no_loop()


def key_released():
    global freeze, show_svg, show_line, show_circle
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    # freeze process, toggle on/off
    if key == "f" or key == "F":
        freeze = not freeze

    # skin style, toggle on/off
    if key == "1":
        show_svg = not show_svg
    if key == "2":
        show_line = not show_line
    if key == "3":
        show_circle = not show_circle


def key_pressed():
    global mouse_rect
    # mouse_rect ctrls arrowkeys up/down
    if key_code == UP:
        mouse_rect += 4
    if key_code == DOWN:
        mouse_rect -= 4


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
