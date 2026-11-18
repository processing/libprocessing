# Cover_Lines
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

# KEYS
# s             : save png
import json
from datetime import datetime

from processing_py import *


class Ribbon:
    def __init__(self, color_points, additional_n, options):
        self.color_points = []
        self.additional_n = additional_n or 0
        for i in range(len(color_points)):
            self.color_points.append(ColorPoint(color_points[i].col))
            # make some more colorPoints to lerp the color
            if i < len(color_points) - 1:
                for j in range(self.additional_n):
                    t = j / self.additional_n
                    col = lerp_color(color_points[i].col, color_points[i + 1].col, t)
                    self.color_points.append(ColorPoint(col))
        self.options = options

    def draw(self, x, y):
        global pen_x, pen_y, pen_a, pen_w
        pen_x = x
        pen_y = y
        pen_a = 0
        pen_w = 50

        for i in range(len(self.color_points)):
            t = remap(i, 0, len(self.color_points) - 1, 0, 1)
            self.color_points[i].draw(t, self.options)


class ColorPoint:
    def __init__(self, col):
        self.col = col
        self.hue_index = 0
        self.alpha_index = 0

    def draw(self, t, opt):
        global pen_x, pen_y, pen_a, pen_w
        # stroke_cap(SQUARE)
        stroke_weight(0.3)
        stroke(self.col)
        no_fill()

        r = red(self.col)
        g = green(self.col)
        b = blue(self.col)
        a = alpha(self.col)

        new_a = pen_a + angle_difference(radians(hue(self.col)), pen_a) * 0.02
        new_x = pen_x + cos(new_a) * opt["step"]
        new_y = pen_y + sin(new_a) * opt["step"]
        new_w = pen_w + (remap(alpha(self.col), 0, 1, opt["minW"], opt["maxW"]) - pen_w) * 0.05

        x1o = pen_x + cos(pen_a - HALF_PI) * pen_w / 2
        y1o = pen_y + sin(pen_a - HALF_PI) * pen_w / 2
        x1u = pen_x + cos(pen_a + HALF_PI) * pen_w / 2
        y1u = pen_y + sin(pen_a + HALF_PI) * pen_w / 2

        x2o = new_x + cos(new_a - HALF_PI) * new_w / 2
        y2o = new_y + sin(new_a - HALF_PI) * new_w / 2
        x2u = new_x + cos(new_a + HALF_PI) * new_w / 2
        y2u = new_y + sin(new_a + HALF_PI) * new_w / 2

        for i in range(opt["n"] + 1):
            t = i / opt["n"]
            x1 = lerp(x1o, x1u, t)
            y1 = lerp(y1o, y1u, t)
            x2 = lerp(x2o, x2u, t)
            y2 = lerp(y2o, y2u, t)

            afac = 1 - 2 * abs(t - 0.5)
            # afac = 1 - pow(2 * (t - 0.5), 2)
            alph = a * afac
            alph = pow(alph, 0.75)

            stroke(r, g, b, alph)
            line(x1, y1, x2, y2)

        pen_x = new_x
        pen_y = new_y
        pen_a = new_a
        pen_w = new_w


def angle_difference(the_angle_1, the_angle_2):
    a1 = (the_angle_1 % TWO_PI + TWO_PI) % TWO_PI
    a2 = (the_angle_2 % TWO_PI + TWO_PI) % TWO_PI

    if a2 > a1:
        d1 = a2 - a1
        d2 = a1 + TWO_PI - a2
        if d1 <= d2:
            return -d1
        else:
            return d2
    else:
        d1 = a1 - a2
        d2 = a2 + TWO_PI - a1
        if d1 <= d2:
            return d1
        else:
            return -d2


# the pen of the ribbon
pen_x = 0
pen_y = 0
pen_a = 0
pen_w = 0

# loaded from data/imageColors.json
image_colors = None

color_points = []
ribbon = None

# chose 8 for print quality (caution: very slow)
mm_to_px = 2

# these are in mm:
cover_width = 800
cover_height = 600


def setup():
    global image_colors, ribbon
    size(cover_width * mm_to_px, cover_height * mm_to_px)

    with open("data/imageColors.json") as f:
        image_colors = json.load(f)

    # create colorPoints
    for i in range(len(image_colors)):
        c = color(image_colors[i]["r"] / 255, image_colors[i]["g"] / 255, image_colors[i]["b"] / 255, image_colors[i]["a"] / 255)
        color_points.append(ColorPoint(c))

    scale(mm_to_px)

    # E Einleitung
    ribbon = Ribbon(color_points[0:41], 300, {"n": 30, "step": 0.15, "damp": 0.02, "minW": 20, "maxW": 100})
    ribbon.draw(250, 200)

    # P.0 p5.js-Einführung
    # ribbon = Ribbon(color_points[41:57], 300, {"n": 30, "step": 0.1, "damp": 0.02, "minW": 20, "maxW": 100})
    # ribbon.draw(250, 50)

    # P.1 Color
    # ribbon = Ribbon(color_points[57:77], 300, {"n": 30, "step": 0.2, "damp": 0.02, "minW": 20, "maxW": 100})
    # ribbon.draw(350, 200)

    # P.2 Shape
    # ribbon = Ribbon(color_points[77:149], 300, {"n": 30, "step": 0.15, "damp": 0.02, "minW": 30, "maxW": 130})
    # ribbon.draw(500, 350)

    # P.3 Type
    # ribbon = Ribbon(color_points[149:187], 300, {"n": 30, "step": 0.1, "damp": 0.02, "minW": 20, "maxW": 100})
    # ribbon.draw(350, 350)

    # P.4 Image
    # ribbon = Ribbon(color_points[187:225], 300, {"n": 30, "step": 0.1, "damp": 0.02, "minW": 20, "maxW": 100})
    # ribbon.draw(600, 150)

    # A Anhang
    # ribbon = Ribbon(color_points[225:258], 300, {"n": 30, "step": 0.1, "damp": 0.02, "minW": 20, "maxW": 100})
    # ribbon.draw(300, 50)

    no_loop()


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
