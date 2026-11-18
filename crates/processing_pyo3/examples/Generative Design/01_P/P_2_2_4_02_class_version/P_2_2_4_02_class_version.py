# P_2_2_4_02
#
# Generative Gestaltung – Creative Coding im Web
# ISBN: 978-3-87439-902-9, First Edition, Hermann Schmidt, Mainz, 2018
# Benedikt Groß, Hartmut Bohnacker, Julia Laub, Claudius Lazzeroni
# with contributions by Joey Lee and Niels Poldervaart
# Copyright 2018
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
# Ported from the p5.js original to processing_py.

# limited diffusion aggregation
#
# KEYS
# 1             : toggle draw original position of circles
# s             : save png
from datetime import datetime

from processing_py import *


class Shape:
    def __init__(self, x, y, r, new_x=None, new_y=None):
        self.x = x
        self.y = y
        self.r = r
        self.new_x = new_x
        self.new_y = new_y


shapes = []
max_count = 5000  # max count of the cirlces
x = []
y = []
r = []
x2 = []
y2 = []

draw_ghosts = False


def setup():
    size(800, 800)

    # first circle
    shapes.append(Shape(width / 2, height / 2, 360))


def draw():
    background(1.0)

    stroke_weight(0.5)
    no_fill()

    # create a random set of parameters
    new_r = random(1, 7)
    new_x = random(new_r, width - new_r)
    new_y = random(new_r, height - new_r)

    closest_dist = float("inf")
    closest_index = 0
    # which shape is the closest?
    closest_shape = None
    for shape in shapes:
        new_dist = dist(new_x, new_y, shape.x, shape.y)
        if new_dist < closest_dist:
            closest_dist = new_dist
            closest_shape = shape

    # align it to the closest circle outline
    angle = atan2(new_y - closest_shape.y, new_x - closest_shape.x)

    shapes.append(Shape(
        closest_shape.x + cos(angle) * (closest_shape.r + new_r),
        closest_shape.y + sin(angle) * (closest_shape.r + new_r),
        new_r,
        new_x,
        new_y,
    ))

    # draw circles at random position and lines
    for index in range(len(shapes)):
        shape = shapes[index]
        # the first circle has no original position
        if draw_ghosts and shape.new_x is not None:
            fill(0.9)
            ellipse(shape.new_x, shape.new_y, shape.r * 2, shape.r * 2)
            line(shape.new_x, shape.new_y, shape.x, shape.y)
        if index == 0:
            no_fill()
        else:
            fill(0.2)
        ellipse(shape.x, shape.y, shape.r * 2, shape.r * 2)

    if len(shapes) >= max_count:
        no_loop()


def key_released():
    global draw_ghosts
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "1":
        draw_ghosts = not draw_ghosts


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
