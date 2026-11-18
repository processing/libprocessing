# P_3_2_5_01
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

# Animated type using loadPixels() method to get font area
#
# MOUSE
# position x/y             : affect randomness
#
# KEYS
# A-Z                      : type letters
# Arrow left/right         : toggle through draw modes
# Arrow up/down            : increase/decrease point density
# CONTROL                  : save png
#
# CONTRIBUTED BY
# [Joey Lee](http://jk-lee.com)
from datetime import datetime

from processing_py import *

font = None

text_typed = "TYPE & CODE"
draw_mode = 1
font_size = 250
padding = 10
n_off = 0
point_density = 8

colors = []

paths = None
text_img = None


def setup():
    global font, colors
    size(1600, 800)
    font = load_font("data/FiraSansCompressed-Bold.otf")
    frame_rate(25)
    rect_mode(CENTER)

    colors = [color(65 / 255, 105 / 255, 185 / 255), color(245 / 255, 95 / 255, 80 / 255), color(15 / 255, 233 / 255, 118 / 255)]

    setup_text()


def setup_text():
    global text_img
    # create an offscreen graphics object to draw the text into
    text_img = create_graphics(width, height)
    text_img.begin_draw()
    text_img.background(1.0)
    text_img.fill(0.0)
    text_img.text_font(font)
    text_img.text_size(font_size)
    text_img.text(text_typed, 100, font_size + 50)
    text_img.end_draw()
    text_img.load_pixels()


def draw():
    global n_off
    background(1.0)

    n_off += 1

    for x in range(0, text_img.width, point_density):
        for y in range(0, text_img.height, point_density):
            # Calculate the index for the pixels array from x and y
            index = x + y * text_img.width
            # Get the red value from image
            r = text_img.pixels[index].r

            if r < 0.5:
                if draw_mode == 1:
                    stroke_weight(1)

                    noise_fac = remap(mouse_x, 0, width, 0, 1)
                    length_fac = remap(mouse_y, 0, height, 0.01, 1)

                    num = noise((x + n_off) * noise_fac, y * noise_fac)
                    if num < 0.6:
                        stroke(colors[0])
                    elif num < 0.7:
                        stroke(colors[1])
                    else:
                        stroke(colors[2])

                    push()
                    translate(x, y)
                    rotate(radians(frame_count))
                    line(0, 0, font_size * length_fac, 0)
                    pop()

                if draw_mode == 2:
                    stroke(0.0, 0.0, 0.0)
                    stroke_weight(1)
                    no_stroke()
                    push()
                    translate(x, y)

                    num = noise((x + n_off) / 10, y / 10)

                    if num < 0.6:
                        fill(colors[0])
                    elif num < 0.7:
                        fill(colors[1])
                    else:
                        fill(colors[2])

                    w = noise((x - n_off) / 10, (y + n_off * 0.1) / 10) * 20
                    h = noise((x - n_off) / 10, (y + n_off * 0.1) / 10) * 10
                    ellipse(0, 0, w, h)  # rect() is cool too
                    pop()

                if draw_mode == 3:
                    stroke(0.0, 0.0, 0.0)
                    stroke_weight(1)
                    no_stroke()

                    num = random(1)

                    if num < 0.6:
                        fill(colors[0])
                    elif num < 0.7:
                        fill(colors[1])
                    else:
                        fill(colors[2])

                    push()
                    begin_shape()
                    for i in range(3):
                        ox = (noise((i * 1000 + x - n_off) / 30, (i * 3000 + y + n_off) / 30) - 0.5) * point_density * 6
                        oy = (noise((i * 2000 + x - n_off) / 30, (i * 4000 + y + n_off) / 30) - 0.5) * point_density * 6
                        vertex(x + ox, y + oy)
                    end_shape(CLOSE)
                    pop()

                if draw_mode == 4:
                    stroke(colors[0])
                    stroke_weight(3)

                    point(x - 10, y - 10)
                    point(x, y)
                    point(x + 10, y + 10)

                    for i in range(5):
                        if i == 1:
                            stroke(colors[1])
                        elif i == 3:
                            stroke(colors[2])

                        if i % 2 == 0:
                            ox = noise((10000 + i * 100 + x - n_off) / 10) * 10
                            oy = noise((20000 + i * 100 + x - n_off) / 10) * 10
                            point(x + ox, y + oy)
                        else:
                            ox = noise((30000 + i * 100 + x - n_off) / 10) * 10
                            oy = noise((40000 + i * 100 + x - n_off) / 10) * 10
                            point(x - ox, y - oy)


def key_pressed():
    global text_typed, draw_mode, point_density
    if key_code == CONTROL:
        save_frame(timestamp() + ".png")

    if key_code == DELETE or key_code == BACKSPACE:
        text_typed = text_typed[0 : max(0, len(text_typed) - 1)]
        setup_text()
    if key_code == ENTER:
        text_typed += "\n"
        setup_text()
    if key_code == LEFT_ARROW:
        draw_mode -= 1
        if draw_mode < 1:
            draw_mode = 4
    if key_code == RIGHT_ARROW:
        draw_mode += 1
        if draw_mode > 4:
            draw_mode = 1
    if key_code == DOWN:
        point_density -= 1
        if point_density < 4:
            point_density = 4
    if key_code == UP:
        point_density += 1


def key_typed():
    global text_typed
    if key is not None and ord(key) >= 32:
        text_typed += key
        setup_text()


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
