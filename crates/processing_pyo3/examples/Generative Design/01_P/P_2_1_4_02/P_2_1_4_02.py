# P_2_1_4_02
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

# Use a video to check on/off a grid of checkboxes
# Shout out to Dan Shiffman's checkbox mirror example
#
# The html checkboxes of the original are drawn on the canvas instead, and its
# slider is not ported; the threshold is the module-level variable below.
#
# CONTRIBUTED BY
# [Joey Lee](http://jk-lee.com)
#
# INSPIRED BY
# [Dan Shiffman](http://shiffman.net/)
from processing_py import *

video = None
# the image threshold (slider of the original, range 0 - 255)
threshold = 200
cols = 40
rows = 40
boxes = []

# size of a checkbox and the space one checkbox takes up in the grid
box_size = 13
box_spacing = 20


def setup():
    global video
    size(cols * box_spacing, rows * box_spacing)

    video = load_movie("data/ball.mov")  # GAP: no video playback; load_movie(path) -> Movie is missing

    video.size(cols, rows)  # GAP: Movie.size(w, h)

    for y in range(rows):
        for x in range(cols):
            box = False
            boxes.append(box)

    # play the video in a loop
    video.loop()  # GAP: Movie.loop()


def draw():
    background(1.0)

    video.load_pixels()  # GAP: Movie.load_pixels()
    for y in range(video.height):  # GAP: Movie.width / Movie.height
        for x in range(video.width):
            # get the video pixel location
            index = x + (y * video.height)
            c = video.pixels[index]  # GAP: Movie.pixels
            r = red(c) * 255
            g = green(c) * 255
            b = blue(c) * 255

            bright = (r + g + b) / 3

            check_index = x + y * cols

            if bright > threshold - 1:
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


run()
