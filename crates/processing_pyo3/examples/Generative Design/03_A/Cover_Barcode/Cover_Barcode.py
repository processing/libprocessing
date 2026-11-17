# Cover_Barcode
#
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
            self.color_points.append(ColorPoint(color_points[i].num, color_points[i].col, color_points[i].fit_t))
            # make some more colorPoints to lerp the color
            if i < len(color_points) - 1:
                for j in range(self.additional_n):
                    t = j / self.additional_n
                    num = lerp(color_points[i].num, color_points[i + 1].num, t)
                    col = lerp_color(color_points[i].col, color_points[i + 1].col, t)
                    fit_t = lerp(color_points[i].fit_t, color_points[i + 1].fit_t, t)
                    self.color_points.append(ColorPoint(num, col, fit_t))
        self.options = options

    def draw(self):
        global pen_x, pen_y, pen_a, pen_w, pen_fit
        pen_x = 0
        pen_y = 0
        pen_a = -HALF_PI
        pen_w = 50
        pen_fit = 0

        for i in range(len(self.color_points)):
            t = remap(i, 0, len(self.color_points) - 1, 0, 1)
            self.color_points[i].draw(t, self.options)

        fill(1.0)
        no_stroke()
        rect(-30, -2, 60, 10)


class ColorPoint:
    def __init__(self, num, col, fit_t):
        self.num = num
        self.col = col
        self.fit_t = fit_t
        self.hue_index = 0
        self.alpha_index = 0

    def draw(self, t_all, opt):
        global pen_x, pen_y, pen_a, pen_w, pen_fit
        # stroke_cap(SQUARE)
        stroke_weight(0.3)
        stroke(0.0)
        no_fill()

        r = red(self.col)
        g = green(self.col)
        b = blue(self.col)
        a = alpha(self.col)

        new_a = pen_a + angle_difference(radians(remap(self.num, 9, 0, 0, 360) - 90), pen_a) * 0.02
        new_x = pen_x + cos(new_a) * opt["step"]
        new_y = pen_y + sin(new_a) * opt["step"]
        new_w = pen_w + (remap(self.num, 0, 9, opt["minW"], opt["maxW"]) - pen_w) * 0.05
        new_fit = pen_fit + (self.fit_t - pen_fit) * 0.02
        x1o = pen_x + cos(pen_a - HALF_PI) * pen_w / 2
        y1o = pen_y + sin(pen_a - HALF_PI) * pen_w / 2
        x1u = pen_x + cos(pen_a + HALF_PI) * pen_w / 2
        y1u = pen_y + sin(pen_a + HALF_PI) * pen_w / 2

        x2o = new_x + cos(new_a - HALF_PI) * new_w / 2
        y2o = new_y + sin(new_a - HALF_PI) * new_w / 2
        x2u = new_x + cos(new_a + HALF_PI) * new_w / 2
        y2u = new_y + sin(new_a + HALF_PI) * new_w / 2

        bw = 184
        bxoff = 12.5
        bs = bw / opt["maxW"]

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

            t = (barcode_info[i]["x"] + barcode_info[i]["width"] / 2 - bxoff) / bw
            bx1 = lerp(x1o, x1u, t)
            by1 = lerp(y1o, y1u, t)
            bx2 = lerp(x2o, x2u, t)
            by2 = lerp(y2o, y2u, t)
            bsw = barcode_info[i]["width"] * bs / bw * 14 * remap(t_all, 0, 1, 1, 0.3)
            balph = lerp(a, alph, t_all)

            fitx1 = lerp(bx1, x1, new_fit)
            fity1 = lerp(by1, y1, new_fit)
            fitx2 = lerp(bx2, x2, new_fit)
            fity2 = lerp(by2, y2, new_fit)
            fitsw = lerp(bsw, 0.3, new_fit)
            fitalph = lerp(balph, alph, new_fit)

            stroke_weight(bsw)
            stroke(r, g, b, balph)
            # stroke(r, g, b)
            line(bx1, by1, bx2, by2)

        pen_x = new_x
        pen_y = new_y
        pen_a = new_a
        pen_w = new_w
        pen_fit = new_fit


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
pen_fit = 0

# loaded from data/barcodeInfo.json and data/imageColors.json
barcode_info = None
image_colors = None

color_points = []
ribbon = None

# chose 8 for print quality (caution: very slow)
mm_to_px = 3

# these are in mm:
cover_width = 400
cover_height = 400

barcode_numbers = [9, 9, 7, 8, 3, 8, 7, 4, 3, 9, 9, 0, 2, 9]


def setup():
    global barcode_info, image_colors, ribbon
    size(cover_width * mm_to_px, cover_height * mm_to_px)

    with open("data/barcodeInfo.json") as f:
        barcode_info = json.load(f)
    with open("data/imageColors.json") as f:
        image_colors = json.load(f)

    # create colorPoints
    for i in range(len(barcode_numbers)):
        n = barcode_numbers[i]
        r = image_colors[n]["r"]
        g = image_colors[n]["g"]
        b = image_colors[n]["b"]
        a = image_colors[n]["a"] * remap(i, 0, len(barcode_numbers) - 1, 1, 0.05)
        col = color(r / 255, g / 255, b / 255, a / 255)

        color_points.append(ColorPoint(n, col, 1))

    # Barcode
    ribbon = Ribbon(color_points, 150, {"n": len(barcode_info) - 1, "step": 0.15, "damp": 0.02, "minW": 10, "maxW": 50})


def draw():
    # background(1.0)
    translate(150 * mm_to_px, 50 * mm_to_px)
    rotate(radians(180))
    scale(mm_to_px)

    ribbon.draw()

    no_loop()


def key_released():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
