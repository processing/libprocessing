# P_2_1_4_03
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

# Sliders arranged in a pattern
#
# The html sliders of the original are drawn on the canvas instead, so they
# cannot be dragged.
#
# CONTRIBUTED BY
# [Joey Lee](http://jk-lee.com)
#
# INSPIRED BY
# [Dan Shiffman](http://shiffman.net/)
from processing_py import *


# stands in for the html slider made by createSlider() in the original
class Slider:
    def __init__(self, min_value, max_value, value):
        self.min_value = min_value
        self.max_value = max_value
        self.value = value
        self.x = 0
        self.y = 0
        self.w = 130
        self.angle = 0

    # x, y is the top left corner of the slider
    def position(self, x, y):
        self.x = x
        self.y = y

    def draw(self):
        knob_size = 15
        push_matrix()
        # like the css transform, the slider rotates around its center
        translate(self.x + self.w / 2, self.y + slider_height / 2)
        rotate(radians(self.angle))
        # track
        stroke(0.72)
        stroke_weight(4)
        line(-self.w / 2, 0, self.w / 2, 0)
        # knob
        knob_x = remap(self.value, self.min_value, self.max_value, -self.w / 2 + knob_size / 2, self.w / 2 - knob_size / 2)
        stroke(0.5)
        stroke_weight(1)
        fill(0.96)
        circle(knob_x, 0, knob_size)
        pop_matrix()


w = 600
h = 600
slider_count = 0
slider_width = 0
slider_height = 17
padding = 10

slider_min = 0
slider_max = 100


def setup():
    global slider_width, slider_count
    size(800, 800)
    # get the number of sliders based on w & h
    slider_width = w / 2
    slider_count = ceil(slider_width / slider_height)

    no_loop()


def draw():
    background(48 / 255, 58 / 255, 118 / 255)

    # topleft - horizontal
    for i in range(slider_count + 1):
        sval = remap(i, slider_count, 0, slider_min, slider_max)
        slider = Slider(slider_min, slider_max, sval)
        slider.position(padding, i * slider_height + padding)
        slider.w = slider_width
        slider.draw()

    # bottomright - horizontal
    for i in range(slider_count + 1):
        sval = remap(i, slider_count, 0, slider_min, slider_max)
        slider = Slider(slider_min, slider_max, sval)
        slider.position(slider_width + padding * 3, slider_width + padding * 2 + i * slider_height)
        slider.w = slider_width
        slider.draw()

    # topright - vertical
    for i in range(slider_count + 1):
        sval = remap(i, 0, slider_count, slider_min, slider_max)
        slider = Slider(slider_min, slider_max, sval)
        slider.position(slider_width / 2 + padding * 2 + i * slider_height, slider_width / 2 + padding)
        slider.w = slider_width
        slider.angle = 90
        slider.draw()

    # bottomleft - vertical
    for i in range(slider_count + 1):
        sval = remap(i, slider_count, 0, slider_min, slider_max)
        slider = Slider(slider_min, slider_max, sval)
        slider.position(slider_width / 2 - i * slider_height + padding * 2, slider_width + slider_width / 2 + padding * 3)
        slider.w = slider_width
        slider.angle = 90
        slider.draw()


run()
