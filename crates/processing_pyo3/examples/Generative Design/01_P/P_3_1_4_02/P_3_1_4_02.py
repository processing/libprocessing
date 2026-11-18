# P_3_1_4_02
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

# Counting the words of a text and display them in a treemap diagram.
# All words with the same number of letters are grouped in a nested treemap.
# Only words with less than 10 letters are stored.
#
# KEYS
# r                   : toggle random mode
# h                   : only horizontal rows
# v                   : only vertical rows
# b                   : both kind of rows
# 1-9                 : switch on and off words with this number of letters
# 0                   : show all words
# s                   : save png
import os
import re
from datetime import datetime

from processing_py import *

joined_text = ""

treemap = None

font = None

do_sort = True
row_direction = "both"


# JavaScript division: x / 0 is Infinity (0 / 0 is NaN) instead of an error
def divide(a, b):
    if b == 0:
        if a > 0:
            return float("inf")
        if a < 0:
            return float("-inf")
        return float("nan")
    return a / b


def get_count(item):
    return item.count


def shuffle_array(array):
    for i in range(len(array) - 1, 0, -1):
        j = floor(random(1) * (i + 1))
        temp = array[i]
        array[i] = array[j]
        array[j] = temp


# Treemap class, ported from the generative-design-library of the p5.js edition.
# A Treemap is either the root (position, size and options are given) or one
# item of its parent (parent, data and count are given).
class Treemap:
    def __init__(self, x, y, w, h, options, parent=None, data=None, count=0):
        self.parent = parent
        self.data = data
        self.count = count
        self.items = []
        # lookup of the items by their data, so add_data() does not have to search the list
        self.item_by_data = {}

        # position and size of the rectangle
        self.x = x
        self.y = y
        self.w = w
        self.h = h

        # the minimum and maximum count value of the items in the items list
        self.min_count = 0
        self.max_count = 0

        # level of the item; the root node has level 0
        if self.parent is not None:
            self.level = self.parent.level + 1
        else:
            self.level = 0

        # the depth of the branch; end nodes have depth 0
        self.depth = 0

        # the number of items in the complete branch
        self.item_count = 1

        # index of the item in the sorted items list
        self.index = 0

        self.root = self
        self.is_root = True
        if self.parent is not None:
            self.root = self.parent.root
            self.is_root = False
        if options is not None:
            self.options = options
        else:
            self.options = self.root.options

        self.ignored = False

    # Adds data to the Treemap. If there is already an item which has this value
    # as data, just increase the counter of that item. If not, create a new
    # Treemap with that data and init the counter with 1.
    def add_data(self, data):
        if data in self.item_by_data:
            # the element is already in this Treemap, so just increase counter
            self.item_by_data[data].count += 1
            return False
        else:
            # the element is not found, so create a new Treemap for it
            t = Treemap(0, 0, 0, 0, None, self, data, 1)
            self.items.append(t)
            self.item_by_data[data] = t
        return True

    # Adds an empty treemap to this treemap. If data is given, this could be used
    # to show and hide a complete sub-treemap from the diagram.
    def add_treemap(self, data, count=0):
        t = Treemap(0, 0, 0, 0, None, self, data, count)
        self.items.append(t)
        return t

    # The size of a rectangle depends on the counter. So it's important to sum
    # up all the counters recursively. Only called internally.
    def sum_up_counters(self):
        # Adjust self.ignored: if the ignore option is defined and self.data is listed there, ignored = True
        if "ignore" in self.options:
            if self.data in self.options["ignore"]:
                self.ignored = True
            else:
                self.ignored = False

        # return count or 0 depending on self.ignored
        if len(self.items) == 0:
            if self.ignored:
                return 0

        else:
            self.min_count = float("inf")
            self.max_count = 0
            self.depth = 0
            self.item_count = 1
            self.count = 0

            if self.ignored:
                return 0

            for i in range(len(self.items)):
                sum = self.items[i].sum_up_counters()
                self.count += sum
                self.min_count = min(self.min_count, sum)
                self.max_count = max(self.max_count, sum)
                self.depth = max(self.depth, self.items[i].depth + 1)
                self.item_count += self.items[i].item_count
        return self.count

    # Calculates the rectangles of each item. While doing this, all counters
    # and ignore flags are updated.
    def calculate(self):
        # Stop immediately, if it's an empty list
        if len(self.items) == 0:
            return

        # if it's the root node, sum up all counters recursively
        if self == self.root:
            self.sum_up_counters()

        # If to ignore this element, adjust parameters and stop
        if self.ignored:
            self.x = -100000  # just a value far outside the screen, so it won't show up if it's drawn accidentally
            self.y = 0
            self.w = 0
            self.h = 0
            return

        # sort or shuffle according to the given option
        if "sort" not in self.options or self.options["sort"] == True:
            # sort items
            self.items.sort(key=get_count, reverse=True)
        else:
            # shuffle explicitly
            shuffle_array(self.items)

        # give every child an index. could be handy for drawing
        for i in range(len(self.items)):
            self.items[i].index = i

        # Starting point is a rectangle and a number of counters to fit in.
        # So, as nothing has fit in the rect, rest_sum, rest_w, ... are the starting rect and the sum of all counters
        rest_sum = self.count
        pad = 0
        if "padding" in self.options:
            pad = self.options["padding"]
        rest_x = self.x + pad
        rest_y = self.y + pad
        rest_w = self.w - pad * 2
        rest_h = self.h - pad * 2

        # Fit in rows. One row consits of one or more rects that should be as square as possible in average.
        # act_index always points on the first counter, that has not fitted in.
        act_index = 0
        while act_index < len(self.items):
            # A row is always along the shorter edge (a).
            is_horizontal = True  # horizontal row
            a = rest_w
            b = rest_h
            if self.options["direction"] != "horizontal":
                if rest_w > rest_h or self.options["direction"] == "vertical":
                    is_horizontal = False  # vertical row
                    a = rest_h
                    b = rest_w

            # How many items to fit into the row?
            row_sum = 0
            row_count = 0
            av_rel_prev = float("inf")
            i = act_index
            while i < len(self.items):
                row_sum += self.items[i].count
                row_count += 1

                # a * b_len is the rect of the row
                percentage = divide(row_sum, rest_sum)
                b_len = b * percentage
                av_rel = divide(a / row_count, b_len)

                # Let's assume it's a horizontal row. The rects are as square as possible,
                # as soon as the average width (a / row_count) gets smaller than the row height (b_len).
                if av_rel < 1 or i == len(self.items) - 1:
                    # Which is better, the actual or the previous fitting?
                    if av_rel_prev < divide(1, av_rel):
                        # previous fitting is better, so revert to that
                        row_sum -= self.items[i].count
                        row_count -= 1
                        b_len = divide(b * row_sum, rest_sum)
                        i -= 1

                    # get the position and length of the row according to is_horizontal (horizontal or not).
                    a_pos = rest_x
                    b_pos = rest_y
                    a_len = rest_w
                    if not is_horizontal:
                        a_pos = rest_y
                        b_pos = rest_x
                        a_len = rest_h

                    # now we can transform the counters between index act_index and i to rects (in fact to treemaps)
                    for j in range(act_index, i + 1):
                        # map a_len according to the value of the counter
                        a_part = divide(a_len * self.items[j].count, row_sum)
                        if is_horizontal:
                            self.items[j].x = a_pos
                            self.items[j].y = b_pos
                            self.items[j].w = a_part
                            self.items[j].h = b_len
                        else:
                            self.items[j].x = b_pos
                            self.items[j].y = a_pos
                            self.items[j].w = b_len
                            self.items[j].h = a_part
                        # negative width or height not allowed
                        self.items[j].w = max(self.items[j].w, 0)
                        self.items[j].h = max(self.items[j].h, 0)

                        # now that the position, width and height is set, it's possible to calculate the nested treemap.
                        self.items[j].calculate()
                        a_pos += a_part

                    # adjust dimensions for the next row
                    if is_horizontal:
                        rest_y += b_len
                        rest_h -= b_len
                    else:
                        rest_x += b_len
                        rest_w -= b_len
                    rest_sum -= row_sum

                    break

                av_rel_prev = av_rel
                i += 1

            act_index = i + 1


def setup():
    global font, joined_text, treemap
    full_screen()

    font = load_font("data/miso-bold.ttf")
    # open() resolves against the cwd, so build the path from this file
    path = os.path.join(os.path.dirname(__file__), "data", "pride_and_prejudice.txt")
    with open(path, encoding="utf-8") as f:
        joined_text = f.read().splitlines()

    joined_text = " ".join(joined_text)
    # If you want to get rid of all number chars too, just uncomment the following line
    # joined_text = re.sub(r"\d+", "", joined_text)
    words = re.findall(r"\w+", joined_text, re.ASCII)

    # create the main treemap
    treemap = Treemap(1, 1, width - 3, height - 3, {
        "sort": do_sort,
        "direction": row_direction,
        "padding": 2,
        "ignore": []
    })

    # make a list for the nested treemaps
    sub_treemaps = [None] * 10

    # count words
    for i in range(len(words)):
        w = words[i].lower()
        index = len(w)
        # Add only words with less than 10 letters
        if index < 10:
            t = sub_treemaps[index]
            if t is None:
                t = treemap.add_treemap(index)
                sub_treemaps[index] = t
            t.add_data(w)

    treemap.calculate()


def draw():
    background(1.0)

    text_align(CENTER, BASELINE)

    stroke_weight(1)

    for i in range(len(treemap.items)):
        sub_treemap = treemap.items[i]
        if not sub_treemap.ignored:
            # h = remap(i, 0, len(treemap.items), 50, 150)

            for j in range(len(sub_treemap.items)):
                item = sub_treemap.items[j]
                # s = remap(sub_treemap.items[j].count, 0, sub_treemap.max_count, 10, 30)

                # fill(hsva(h, s / 100, 1.0))
                # stroke(hsva(h, (s + 20) / 100, 0.9))
                no_fill()
                stroke(0.0)
                rect(item.x, item.y, item.w, item.h)

                word = sub_treemap.items[j].data
                text_font(font)
                text_size(100)
                text_w = text_width(word)
                font_size = 100 * (item.w * 0.9) / text_w
                font_size = min(font_size, (item.h * 0.9))
                text_font(font)
                text_size(font_size)

                fill(0.0)
                no_stroke()
                text(word, item.x + item.w / 2, item.y + item.h * 0.8)

    no_loop()


def key_typed():
    global do_sort, row_direction
    # export png
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "r" or key == "R":
        do_sort = not do_sort
        treemap.options["sort"] = do_sort
        treemap.calculate()
        loop()
    if key == "h" or key == "H":
        row_direction = "horizontal"
        treemap.options["direction"] = row_direction
        treemap.calculate()
        loop()
    if key == "v" or key == "V":
        row_direction = "vertical"
        treemap.options["direction"] = row_direction
        treemap.calculate()
        loop()
    if key == "b" or key == "B":
        row_direction = "both"
        treemap.options["direction"] = row_direction
        treemap.calculate()
        loop()

    # number key 1 - 9
    if key is not None and key >= "1" and key <= "9":
        num = int(key)
        # search for the pressed number in the ignore list
        if num in treemap.options["ignore"]:
            # found value, so remove it
            treemap.options["ignore"].remove(num)
        else:
            # not found, so add to list
            treemap.options["ignore"].append(num)
        treemap.calculate()
        loop()
    if key == "0":
        treemap.options["ignore"] = []
        treemap.calculate()
        loop()


def key_pressed():
    # if key_code == RIGHT_ARROW:
    pass


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
