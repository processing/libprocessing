# P_2_1_4_04
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

# Create animated radial sliders
#
# The html sliders of the original are drawn on the canvas instead.
#
# MOUSE
# left click               : create SliderRose
#
# CONTRIBUTED BY
# [Joey Lee](http://jk-lee.com)
#
# INSPIRED BY
# [Dan Shiffman](http://shiffman.net/)
from processing_py import *

# height of an html slider
slider_height = 16


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


# define a SliderRose object
class SliderRose:
    def __init__(self, _x, _y):
        self.x1 = _x
        self.y1 = _y
        # collect the sliders in an array
        self.sliders = []
        self.sin_angle = 0
        # set the slider width
        rose_radius = random(20, 100)
        # define how many degrees to skip from 360
        skip = 20
        # create sliders around a circle
        for i in range(0, 360, skip):
            slider_angle = radians(i)
            x2 = cos(slider_angle) * rose_radius
            y2 = sin(slider_angle) * rose_radius
            # create the slider, position, and rotate
            slider = Slider(0, 255, 50)
            slider.position(self.x1 + x2, self.y1 + y2)
            slider.w = rose_radius
            slider.angle = i
            self.sliders.append(slider)

    # for each loop through the draw function
    # update the sliders according to a sin wave
    def update(self):
        offset = 0
        for i in range(len(self.sliders)):
            # map the value along the sine wave to the slider values
            x = remap(sin(self.sin_angle + offset), -1, 1, 0, 255)
            self.sliders[i].value = x
            self.sliders[i].draw()
            offset += 0.050
        self.sin_angle += 0.1


sliders = []


def setup():
    full_screen()
    # init canvas with slider rose to the middle
    sliders.append(SliderRose(width / 2, height / 2))


def draw():
    background(101 / 255, 179 / 255, 109 / 255)
    # create slider animations
    for d in sliders:
        d.update()


# add a new slider rose when mouse pressed
def mouse_pressed():
    sliders.append(SliderRose(mouse_x, mouse_y))


run()
