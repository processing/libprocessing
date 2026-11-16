# P_4_3_4_01_emoji_icon_analyser
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

# Analyses RGB pixel values of emoji icons to deduce the average colors.
# The results are written in a json file.
import csv
import json

from processing_py import *

emoji_names = None
counter = 0
img_size = 72
results = {}

# You will have to download the emoji files first
# Please see https://github.com/generative-design/Code-Package-p5.js/tree/master/data
emojis_path = "data/twemoji/"


def preload():
    global emoji_names
    # a csv table with a header row
    emoji_names = []
    with open("data/emoji-names.csv", newline="") as f:
        reader = csv.reader(f)
        next(reader)
        for row in reader:
            emoji_names.append(row)


def setup():
    size(img_size, img_size, transparent=True)  # a transparent canvas, like the p5 one, so clear() leaves alpha 0
    preload()
    print(len(emoji_names))


def draw():
    global counter
    if counter >= len(emoji_names):
        with open("emoji-average-colors.json", "w") as f:
            json.dump(results, f, indent=2)
        print("done")
        no_loop()
        return

    current_emoji = emoji_names[counter][0]
    print(current_emoji)
    img = load_image(emojis_path + "72x72/" + current_emoji + ".png")
    clear()
    image(img, 0, 0)
    pixels = load_pixels()
    rgb = {"r": 0, "g": 0, "b": 0}
    pix_counter = 0
    for i in range(len(pixels)):
        if alpha(pixels[i]) == 0:
            continue  # skip if alpha == 0
        rgb["r"] += red(pixels[i]) * 255
        rgb["g"] += green(pixels[i]) * 255
        rgb["b"] += blue(pixels[i]) * 255
        pix_counter += 1
    # int() used to floor values
    rgb["r"] = int(rgb["r"] / pix_counter)
    rgb["g"] = int(rgb["g"] / pix_counter)
    rgb["b"] = int(rgb["b"] / pix_counter)

    results[current_emoji] = {}
    results[current_emoji]["averageColor"] = rgb

    # no_stroke()
    # fill(rgb["r"] / 255, rgb["g"] / 255, rgb["b"] / 255)
    # ellipse(width / 2, height / 2, 30, 30)
    counter += 1


run()
