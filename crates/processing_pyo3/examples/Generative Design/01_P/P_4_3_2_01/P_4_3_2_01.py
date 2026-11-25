# P_4_3_2_01
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

# pixel mapping. each pixel is translated into a new element (letter)
#
# KEYS
# 1                 : toogle font size mode (dynamic/static)
# 2                 : toogle font color mode (color/b&w)
# arrow up/down     : maximal fontsize +/-
# arrow right/left  : minimal fontsize +/-
# s                 : save png
from datetime import datetime

from processing_py import *

input_text = "Ihr naht euch wieder, schwankende Gestalten, Die früh sich einst dem trüben Blick gezeigt. Versuch ich wohl, euch diesmal festzuhalten? Fühl ich mein Herz noch jenem Wahn geneigt? Ihr drängt euch zu! nun gut, so mögt ihr walten, Wie ihr aus Dunst und Nebel um mich steigt; Mein Busen fühlt sich jugendlich erschüttert Vom Zauberhauch, der euren Zug umwittert. Ihr bringt mit euch die Bilder froher Tage, Und manche liebe Schatten steigen auf; Gleich einer alten, halbverklungnen Sage Kommt erste Lieb und Freundschaft mit herauf; Der Schmerz wird neu, es wiederholt die Klage."
font_size_max = 20
font_size_min = 10
spacing = 12  # line height
kerning = 0.5  # between letters

font_size_static = False
black_and_white = False

font = None
img = None


def setup():
    global font, img
    size(533, 769)

    font = create_font("Times")

    img = load_image("data/pic.png")
    print(str(img.width) + " x " + str(img.height))


def draw():
    background(1.0)
    text_align(LEFT)
    # text_align(LEFT, CENTER)  # also nice!

    x = 0
    y = 10
    counter = 0

    while y < height:
        # translate position (display) to position (image)
        img_x = int(remap(x, 0, width, 0, img.width))
        img_y = int(remap(y, 0, height, 0, img.height))
        # get current color
        c = img.pixels[img_y * img.width + img_x]
        greyscale = round(red(c) * 255 * 0.222 + green(c) * 255 * 0.707 + blue(c) * 255 * 0.071)

        push_matrix()
        translate(x, y)

        if font_size_static:
            text_font(font)
            text_size(font_size_max)
            if black_and_white:
                fill(greyscale / 255)
            else:
                fill(c)
        else:
            # greyscale to fontsize
            font_size = remap(greyscale, 0, 255, font_size_max, font_size_min)
            font_size = max(font_size, 1)
            text_font(font)
            text_size(font_size)
            if black_and_white:
                fill(0.0)
            else:
                fill(c)

        letter = input_text[counter]
        text(letter, 0, 0)
        letter_width = text_width(letter) + kerning
        # for the next letter ... x + letter width
        x = x + letter_width  # update x-coordinate
        pop_matrix()

        # linebreaks
        if x + letter_width >= width:
            x = 0
            y = y + spacing  # add line height

        counter += 1
        if counter > len(input_text) - 1:
            counter = 0


def key_released():
    global font_size_static, black_and_white
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    # change render mode
    if key == "1":
        font_size_static = not font_size_static
    # change color stlye
    if key == "2":
        black_and_white = not black_and_white
    print("fontSizeMin: " + str(font_size_min) + "  fontSizeMax: " + str(font_size_max) + "   fontSizeStatic: " + str(font_size_static) + "   blackAndWhite: " + str(black_and_white))


def key_pressed():
    global font_size_max, font_size_min
    # change fontSizeMax with arrowkeys up/down
    if key_code == UP:
        font_size_max += 2
    if key_code == DOWN:
        font_size_max -= 2
    # change fontSizeMin with arrowkeys left/right
    if key_code == RIGHT_ARROW:
        font_size_min += 2
    if key_code == LEFT_ARROW:
        font_size_min -= 2

    # font_size_min = max(font_size_min, 2)
    # font_size_max = max(font_size_max, 2)


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
