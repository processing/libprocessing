# P_2_3_7_01
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

# Simple drawing tool where mouse input gets mirrored over multiple axes
#
# MOUSE
# left click          : draw line
#
# KEYS
# 1                   : toggle vertical mirror
# 2                   : toggle horizontal mirror
# 3                   : toggle diagonal mirror 1
# 4                   : toggle diagonal mirror 2
# 5-9                 : change color
# 0                   : color white (eraser)
# arrow up            : increase line weight
# arrow down          : decrease line weight
# arrow right         : increase number of tiles
# arrow left          : decrease number of tiles
# d                   : show/hide mirror axes
# del, backspace      : clear screen
# s                   : save png
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
from datetime import datetime

from processing_py import *

line_width = 3
line_color = None

mv = True
mh = True
md1 = True
md2 = True
pen_count = 1

show_axes = True

img = None


def setup():
    global line_color, img
    # Please work with a square canvas
    size(800, 800)
    no_cursor()
    no_fill()
    line_color = color(0.0)

    # Create an offscreen graphics object to draw into
    img = create_graphics(width, height)
    img.pixel_density(1)  # GAP: Graphics.pixel_density(density) setter (pixel_density is a read-only property on Graphics)

    background(1.0)


def draw():
    background(1.0)
    image(img, 0, 0)

    img.begin_draw()
    img.stroke_weight(line_width)
    img.stroke(line_color)

    if mouse_is_pressed and mouse_button == LEFT:
        w = width / pen_count
        h = height / pen_count
        x = mouse_x % w
        y = mouse_y % h
        px = x - (mouse_x - pmouse_x)
        py = y - (mouse_y - pmouse_y)

        for i in range(pen_count):
            for j in range(pen_count):
                ox = i * w
                oy = j * h

                # Normal position
                img.line(x + ox, y + oy, px + ox, py + oy)
                # Horizontal mirror or all three other mirrors
                if mh or md2 and md1 and mv:
                    img.line(w - x + ox, y + oy, w - px + ox, py + oy)
                # Vertical mirror
                if mv or md2 and md1 and mh:
                    img.line(x + ox, h - y + oy, px + ox, h - py + oy)
                # Horizontal and vertical mirror
                if mv and mh or md2 and md1:
                    img.line(w - x + ox, h - y + oy, w - px + ox, h - py + oy)

                # When mirroring diagonally, flip X and Y inputs.
                if md1 or md2 and mv and mh:
                    img.line(y + ox, x + oy, py + ox, px + oy)
                if md1 and mh or md2 and mv:
                    img.line(y + ox, w - x + oy, py + ox, w - px + oy)
                if md1 and mv or md2 and mh:
                    img.line(h - y + ox, x + oy, h - py + ox, px + oy)
                if md1 and mv and mh or md2:
                    img.line(h - y + ox, w - x + oy, h - py + ox, w - px + oy)
    img.end_draw()

    if show_axes:
        w = width / pen_count
        h = height / pen_count

        # draw mirror axes and tiles
        for i in range(pen_count):
            for j in range(pen_count):
                x = i * w
                y = j * h

                stroke(0.0, 50 / 255)
                stroke_weight(1)
                if mh:
                    line(x + w / 2, y, x + w / 2, y + h)
                if mv:
                    line(x, y + h / 2, x + w, y + h / 2)
                if md1:
                    line(x, y, x + w, y + h)
                if md2:
                    line(x + w, y, x, y + h)

                stroke(15 / 255, 233 / 255, 118 / 255, 50 / 255)
                stroke_weight(1)
                rect(i * w, j * h, w - 1, h - 1)

        # draw pen
        fill(line_color)
        no_stroke()
        ellipse(mouse_x, mouse_y, line_width + 2, line_width + 2)
        stroke(0.0, 50 / 255)
        no_fill()
        ellipse(mouse_x, mouse_y, line_width + 1, line_width + 1)


def key_pressed():
    global pen_count, line_width, mv, mh, md1, md2, line_color, show_axes
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key_code == DELETE or key_code == BACKSPACE:
        img.begin_draw()
        img.clear()
        img.end_draw()

    if key_code == RIGHT_ARROW:
        pen_count += 1
    if key_code == LEFT_ARROW:
        pen_count = max(1, pen_count - 1)

    if key_code == UP:
        line_width += 1
    if key_code == DOWN:
        line_width = max(1, line_width - 1)

    if key == "1":
        mv = not mv
    if key == "2":
        mh = not mh
    if key == "3":
        md1 = not md1
    if key == "4":
        md2 = not md2

    if key == "5":
        line_color = color(0.0)
    if key == "6":
        line_color = color(15 / 255, 233 / 255, 118 / 255)
    if key == "7":
        line_color = color(245 / 255, 95 / 255, 80 / 255)
    if key == "8":
        line_color = color(65 / 255, 105 / 255, 185 / 255)
    if key == "9":
        line_color = color(1.0, 231 / 255, 108 / 255)
    if key == "0":
        line_color = color(1.0)

    if key == "d" or key == "D":
        show_axes = not show_axes


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
