# P_2_1_4_01
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

# Use an image to check on/off a grid of checkboxes
# Shout out to Dan Shiffman's checkbox mirror example
# Image credits: Map tiles by Stamen Design, under CC BY 3.0. Data by OpenStreetMap, under ODbL.
#
# The html checkboxes of the original are drawn on the canvas instead, and its
# slider is not ported; the threshold is the module-level variable below.
#
# KEYS
# 1                   : load image shapes
# 2                   : load image letters
# 3                   : load image map
#
# CONTRIBUTED BY
# [Joey Lee](http://jk-lee.com)
#
# INSPIRED BY
# [Dan Shiffman](http://shiffman.net/)
from processing_py import *

img = None
img1 = None
img2 = None
img3 = None
# the image threshold (slider of the original, range 0 - 255)
threshold = 0
cols = 40
rows = 40
boxes = []

# size of a checkbox and the space one checkbox takes up in the grid
box_size = 13
box_spacing = 20


def setup():
    global img, img1, img2, img3
    size(cols * box_spacing, rows * box_spacing)

    # load the images to be used for the checkboxes
    img1 = load_image("data/shapes.png")
    img2 = load_image("data/draw.png")
    img3 = load_image("data/toner.png")

    # set the current img
    img = img1
    img.resize(cols, rows)
    img.load_pixels()

    for y in range(rows):
        for x in range(cols):
            box = False
            boxes.append(box)


def draw():
    background(1.0)

    for y in range(img.height):
        for x in range(img.height):
            c = img.get(x, y)
            bright = (red(c) + green(c) + blue(c)) / 3 * 255

            check_index = x + y * cols

            if bright > threshold:
                boxes[check_index] = False
            else:
                boxes[check_index] = True

    # draw the grid of checkboxes
    for y in range(rows):
        for x in range(cols):
            checkbox(x * box_spacing + 4, y * box_spacing + 3, boxes[x + y * cols])


# draw a checkbox with its top left corner at x, y
def checkbox(x, y, checked):
    if checked:
        no_stroke()
        fill(0.45)
        rect(x, y, box_size, box_size, 3)
        # the check mark
        no_fill()
        stroke(1.0)
        stroke_weight(1.5)
        line(x + 3, y + 7, x + 5.5, y + 9.5)
        line(x + 5.5, y + 9.5, x + 10, y + 3.5)
    else:
        fill(1.0)
        stroke(0.7)
        stroke_weight(1)
        rect(x, y, box_size, box_size, 3)


def key_pressed():
    global img
    if key == "1":
        img = img1
    if key == "2":
        img = img2
    if key == "3":
        img = img3

    img.resize(cols, rows)
    img.load_pixels()


run()
