# P_4_2_1_02
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

# radial collage generator. example footage can be found in "data".
# if you use your own footage, make sure to rename the files or adjust the prefixes:
# see the parameters of generate_collage_items()
#
# KEYS
# 1-3                  : create a new collage (layer specific)
# s                    : save png
import os
from datetime import datetime

from processing_py import *


# ------ collage class/record ------
class CollageItem:
    def __init__(self):
        # polar koordinates
        self.a = 0
        self.l = 0
        self.rotation = 0
        self.scaling = 1
        self.index_to_image = -1


images = []
image_names = []
image_count = 0

layer1_items = []
layer2_items = []
layer3_items = []


def setup():
    global images, image_names, image_count
    global layer1_items, layer2_items, layer3_items
    size(1024, 768)
    image_mode(CENTER)
    background(1.0)

    # ------ load images ------
    # replace this location with a folder on your machine
    dir = "data"

    if os.path.isdir(dir):
        contents = sorted(os.listdir(dir))
        images = [None] * len(contents)
        image_names = [None] * len(contents)
        for i in range(len(contents)):
            # skip hidden files and folders starting with a dot, load .png files only
            if contents[i][0] == ".":
                continue
            elif contents[i].lower().endswith(".png"):
                child_file = os.path.join(dir, contents[i])
                images[image_count] = load_image(child_file)
                image_names[image_count] = contents[i]
                print(str(image_count) + " " + contents[i] + "  " + child_file)
                image_count += 1

    # ------ init ------
    # generate_collage_items(filename prefix, count, angle,distance, angle range,distance range, scale start,scale end, rotation start,rotation end)
    # filname prefix               : Alle Bilder, deren Name so beginnt, werden für diesen Layer verwendet
    # count                        : Anzahl der Bilder
    # angle                        : Winkel, um die sich die Bilder scharen
    # distance                     : Distanz vom Mittelpunkt, um die sich die Bilder scharen
    # angle range                  : Winkelbereich, in dem die Bilder gestreut werden (z.B. TWO_PI/3 füllt einen Drittel-Kreis)
    # distance range               : Bereich für die Distanz, in der die Bilder gestreut werden (z.B. distance=300 und distanceRange=100 streut die Positionen in einen Ring von 100 Pixel Dicke)
    # scale start, scale end       : Minimaler und maximaler Wert für den zufälligen Skalierungsfaktor
    # rotation start, rotation end : Minimaler und maximaler Wert für den zufälligen Rotationswinkel

    layer1_items = generate_collage_items("layer1", 100, 0, height / 2, TWO_PI, height, 0.1, 0.5, 0, 0)
    layer2_items = generate_collage_items("layer2", 150, 0, height / 2, TWO_PI, height, 0.1, 0.3, -PI / 6, PI / 6)
    layer3_items = generate_collage_items("layer3", 110, 0, height / 2, TWO_PI, height, 0.1, 0.2, 0, 0)

    # draw collage
    draw_collage_items(layer1_items)
    draw_collage_items(layer2_items)
    draw_collage_items(layer3_items)


def draw():
    # keep the programm running
    pass


# ------ interactions and generation of the collage ------
def key_pressed():
    global layer1_items, layer2_items, layer3_items
    if key == "s" or key == "S":
        save_frame(timestamp() + "_####.png")

    if key == "1":
        layer1_items = generate_collage_items("layer1", int(random(50, 200)), 0, height / 2, PI * 5, height, 0.1, 0.5, 0, 0)
    if key == "2":
        layer2_items = generate_collage_items("layer2", int(random(25, 300)), 0, height * 0.15, PI * 5, 150, 0.1, random(0.3, 0.8), -PI / 6, PI / 6)
    if key == "3":
        layer3_items = generate_collage_items("layer3", int(random(50, 300)), 0, height * 0.66, PI * 5, height * 0.66, 0.1, random(0.2, 0.5), -0.05, 0.05)

    # draw collage
    background(1.0)
    draw_collage_items(layer1_items)
    draw_collage_items(layer2_items)
    draw_collage_items(layer3_items)


# ------ collage items helper functions ------
def generate_collage_items(the_prefix, the_count, the_angle, the_length, the_range_a, the_range_l, the_scale_start, the_scale_end, the_rotation_start, the_rotation_end):
    # collect all images with the specified prefix
    indexes = []
    for i in range(len(image_names)):
        if image_names[i] != None:
            if image_names[i].startswith(the_prefix):
                indexes.append(i)

    items = [None] * the_count
    for i in range(len(items)):
        items[i] = CollageItem()
        items[i].index_to_image = indexes[i % len(indexes)]
        items[i].a = the_angle + random(-the_range_a / 2, the_range_a / 2)
        items[i].l = the_length + random(-the_range_l / 2, the_range_l / 2)
        items[i].scaling = random(the_scale_start, the_scale_end)
        items[i].rotation = items[i].a + HALF_PI + random(the_rotation_start, the_rotation_end)
    return items


def draw_collage_items(the_items):
    for i in range(len(the_items)):
        push_matrix()
        new_x = width / 2 + cos(the_items[i].a) * the_items[i].l
        new_y = height / 2 + sin(the_items[i].a) * the_items[i].l
        translate(new_x, new_y)
        rotate(the_items[i].rotation)
        scale(the_items[i].scaling)
        image(images[the_items[i].index_to_image], 0, 0)
        pop_matrix()


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
