# P_3_1_3_05
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

# analysing and sorting the words of a text by Part of Speech
# connecting subsequent letters with lines.
#
# MOUSE
# position x          : interpolate between normal text and sorted position
#
# KEYS
# 1                   : toggle grey lines on/off
# 2                   : toggle colored lines on/off
# 3                   : toggle text on/off
# 4                   : switch all letters off
# 5                   : switch all letters on
# a-z                 : switch letter on/off
# ctrl                : save png
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
# Adapted from P_3_1_3_04
import os
import re
from datetime import datetime

from processing_py import *

joined_text = []
text_pos_tags = []

# Alphabetical list of PENN part-of-speech tags
# source: https://rednoise.org/rita/reference/PennTags.html
all_pos_tags = [
    "cc",
    "cd",
    "dt",
    "ex",
    "fw",
    "in",
    "jj",
    "jjr",
    "jjs",
    "ls",
    "md",
    "nn",
    "nns",
    "nnp",
    "nnps",
    "pdt",
    "pos",
    "prp",
    "prp$",
    "rb",
    "rbr",
    "rbs",
    "rp",
    "sym",
    "to",
    "uh",
    "vb",
    "vbd",
    "vbg",
    "vbn",
    "vbp",
    "vbz",
    "wdt",
    "wp",
    "wp$",
    "wrb",
]
all_pos_tags_full = [
    "Coordinating conjunction",
    "Cardinal number",
    "Determiner",
    "Existential there",
    "Foreign word",
    "Preposition or subordinating conjunction",
    "Adjective",
    "Adjective, comparative",
    "Adjective, superlative",
    "List item marker",
    "Modal",
    "Noun, singular or mass",
    "Noun, plural",
    "Proper noun, singular",
    "Proper noun, plural",
    "Predeterminer",
    "Possessive ending",
    "Personal pronoun",
    "Possessive pronoun",
    "Adverb",
    "Adverb, comparative",
    "Adverb, superlative",
    "Particle",
    "Symbol",
    "to",
    "Interjection",
    "Verb, base form",
    "Verb, past tense",
    "Verb, gerund or present participle",
    "Verb, past participle",
    "Verb, non-3rd person singular present",
    "Verb, 3rd person singular present",
    "Wh-determiner",
    "Wh-pronoun",
    "Possessive wh-pronoun",
    "Wh-adverb",
]

counters = []

pos_x = 0
pos_y = 0

draw_grey_lines = False
draw_colored_lines = True
draw_text = True


def setup():
    global joined_text
    full_screen()

    # open() resolves against the cwd, so build the path from this file
    path = os.path.join(os.path.dirname(__file__), "data", "AllTheWorldsAStage.txt")
    with open(path, encoding="utf-8") as f:
        joined_text = f.read().splitlines()

    text_font(create_font("monospace"))  # GAP: create_font() does not resolve generic family names such as "monospace" (Font not found)
    text_size(18)
    fill(0.0)

    for i in range(len(all_pos_tags)):
        counters.append(0)

    joined_text = " ".join(joined_text)
    joined_text = re.split(r"\s+", joined_text)
    for i in range(len(joined_text)):

        word_pos_tag = RiTa.get_pos_tags(RiTa.strip_punctuation(joined_text[i]))[0]  # GAP: RiTa (the part-of-speech tagger library used by the p5.js edition) has no counterpart

        text_pos_tags.append(word_pos_tag)

        if word_pos_tag in all_pos_tags:
            tag_index = all_pos_tags.index(word_pos_tag)
            counters[tag_index] += 1

        joined_text[i] += " "


def draw():
    global pos_x, pos_y
    background(1.0)

    translate(50, 0)

    no_stroke()

    pos_x = 0
    pos_y = 50
    sort_positions_x = [0] * len(joined_text)
    old_positions_x = [0] * len(joined_text)
    old_positions_y = [0] * len(joined_text)
    old_x = 0
    old_y = 0

    # draw counters
    if mouse_x >= width - 50:
        text_size(10)
        for i in range(len(all_pos_tags)):
            text_align(LEFT)
            text(all_pos_tags[i] + " (" + all_pos_tags_full[i] + ")", -20, i * 20 + 40)
            text_align(RIGHT)
            text(str(counters[i]), -25, i * 20 + 40)
        text_align(LEFT)
        text_size(18)

    translate(256, 0)

    # go through all characters in the text to draw them
    for i in range(len(joined_text)):
        # again, find the index of the current letter in the alphabet
        word_pos_tag = text_pos_tags[i]
        if word_pos_tag not in all_pos_tags:
            continue
        index = all_pos_tags.index(word_pos_tag)

        m = remap(mouse_x, 50, width - 50, 0, 1)
        m = constrain(m, 0, 1)

        sort_x = sort_positions_x[index]
        inter_x = lerp(pos_x, sort_x, m)

        sort_y = index * 20 + 40
        inter_y = lerp(pos_y, sort_y, m)

        if draw_grey_lines:
            if old_x != 0 and old_y != 0:
                stroke(0.0, 0.1)
                line(old_x, old_y, inter_x, inter_y)
            old_x = inter_x
            old_y = inter_y

        if draw_colored_lines:
            if old_positions_x[index] != 0 and old_positions_y[index] != 0:
                stroke(hsva(index * 10, 0.8, 0.6, 0.5))
                line(old_positions_x[index], old_positions_y[index], inter_x, inter_y)
            old_positions_x[index] = inter_x
            old_positions_y[index] = inter_y

        if draw_text:
            text(joined_text[i], inter_x, inter_y)

        sort_positions_x[index] += text_width(joined_text[i])
        pos_x += text_width(joined_text[i])
        if pos_x >= min(width, 1000):
            pos_y += 40
            pos_x = 0


def key_released():
    global draw_grey_lines, draw_colored_lines, draw_text
    if key_code == CONTROL:
        save_frame(timestamp() + "_##.png")

    if key == "1":
        draw_grey_lines = not draw_grey_lines
    if key == "2":
        draw_colored_lines = not draw_colored_lines
    if key == "3":
        draw_text = not draw_text


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
