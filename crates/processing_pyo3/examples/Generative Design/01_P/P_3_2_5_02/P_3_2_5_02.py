# P_3_2_5_02
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

# Animated font created from simple geometries
#
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


# the AniLetters object defines the simple geometry font
class AniLetters:
    def __init__(self, _lwidth, _lheight):
        self.text_typed = []
        self.paths = []
        self.letter_width = _lwidth
        self.letter_height = _lheight
        self.line_count = 0
        self.ani_steps = 20
        self.draw_mode = 3
        self.cursor_location = {"x": 50, "y": 50}
        self.letter_padding = 60
        self.style = 1

        # the function that draws each letter
        self.letters = {
            " ": self.space,
            "A": self.A,
            "B": self.B,
            "C": self.C,
            "D": self.D,
            "E": self.E,
            "F": self.F,
            "G": self.G,
            "H": self.H,
            "I": self.I,
            "J": self.J,
            "K": self.K,
            "L": self.L,
            "M": self.M,
            "N": self.N,
            "O": self.O,
            "P": self.P,
            "Q": self.Q,
            "R": self.R,
            "S": self.S,
            "T": self.T,
            "U": self.U,
            "V": self.V,
            "W": self.W,
            "X": self.X,
            "Y": self.Y,
            "Z": self.Z,
        }

    #
    # Data Handling
    #

    # set the line_count to the number of "lines" or text object in the text_typed list
    def get_line_count(self):
        if len(self.text_typed) > 0:
            self.line_count = len(self.text_typed) - 1
        else:
            self.line_count = 0

    # get each data path of the letters
    def get_paths(self):
        self.paths = []

        for idx in range(len(self.text_typed)):
            txt = self.text_typed[idx]
            for i in range(len(txt["text"])):
                d = txt["text"][i]

                path_data = {
                    "letter": d.upper(),
                    "x": self.cursor_location["x"] + (self.letter_width + self.letter_padding * i),
                    "y": self.cursor_location["y"] + (self.letter_height * idx),
                }

                self.paths.append(path_data)

    # add a text object for each line
    def add_text(self, _text):
        text_object = {"counter": 0, "text": _text}
        return text_object

    #
    # Keyboard interactions
    #

    # remove letters
    def remove_letters(self):
        text_typed_counter = self.line_count
        # remove letters from each object
        if text_typed_counter >= 0 and len(self.text_typed[0]["text"]) > 0:
            line_text = self.text_typed[text_typed_counter]["text"]
            self.text_typed[text_typed_counter]["text"] = line_text[0 : max(0, len(line_text) - 1)]
        # remove objects if there's no characters
        if len(self.text_typed[text_typed_counter]["text"]) == 0:
            text_typed_counter -= 1
            if text_typed_counter < 0:
                print("nothing left")
                text_typed_counter = 0
            else:
                self.text_typed.pop()

    # add lines
    def add_lines(self):
        self.text_typed.append(self.add_text(""))
        self.line_count += 1

    # add characters
    def add_characters(self, _key):
        if _key.upper() in self.letters:
            self.text_typed[self.line_count]["text"] += _key
        else:
            print("not a letter")

    #
    # Call functions in render
    #

    def render(self):
        if len(self.paths) > 0:
            for d in self.paths:
                self.letters[d["letter"]](d["x"], d["y"])

    #
    # Letter Definitions
    #

    # space
    def space(self, x, y):
        # nothing
        pass

    def A(self, x, y):
        push()
        translate(x, y)
        self.diagonal_to_middle(self.letter_width / 2, 0, 1)
        self.diagonal_to_middle(-self.letter_width / 2, 0, -1)
        self.half_cross_bar(self.letter_width / 4, self.letter_height / 2)
        pop()

    def B(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.half_bowl(0, 0, -1)
        self.half_bowl(0, self.letter_height / 2, -1)
        pop()

    def C(self, x, y):
        push()
        translate(x, y)
        self.full_bowl(0, 0, 1)
        pop()

    def D(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.full_bowl(0, 0, -1)
        pop()

    def E(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.cross_bar(0, 0)
        self.cross_bar(0, self.letter_height / 2)
        self.cross_bar(0, self.letter_height)
        pop()

    def F(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.cross_bar(0, 0)
        self.cross_bar(0, self.letter_height / 2)
        pop()

    def G(self, x, y):
        push()
        translate(x, y)
        self.full_bowl(0, 0, 1)
        self.half_stem(self.letter_width, self.letter_height / 2)
        self.half_cross_bar(self.letter_width / 2, self.letter_height / 2)
        pop()

    def H(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.cross_bar(0, self.letter_height / 2)
        self.full_stem(self.letter_width, 0)
        pop()

    def I(self, x, y):
        push()
        translate(x, y)
        self.full_stem(self.letter_width / 2, 0)
        pop()

    def J(self, x, y):
        push()
        translate(x, y)
        self.j_curve(0, 0)
        pop()

    def K(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.half_diagonal_leg(0, self.letter_height / 2, 1)
        self.half_diagonal_leg(0, self.letter_height / 2, -1)
        pop()

    def L(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.cross_bar(0, self.letter_height)
        pop()

    def M(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.full_stem(self.letter_width, 0)
        self.diagonal_to_middle(0, 0, 1)
        self.diagonal_to_middle(0, 0, -1)
        pop()

    def N(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.full_stem(self.letter_width, 0)
        self.diagonal_to_end(0, 0, -1)
        pop()

    def O(self, x, y):
        push()
        translate(x, y)
        self.letter_o(0, 0)
        pop()

    def P(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.half_bowl(0, 0, -1)
        pop()

    def Q(self, x, y):
        push()
        translate(x, y)
        self.letter_o(0, 0)
        self.half_diagonal_arm(self.letter_width / 2, self.letter_height / 2, -1)
        pop()

    def R(self, x, y):
        push()
        translate(x, y)
        self.full_stem(0, 0)
        self.half_bowl(0, 0, -1)
        self.half_diagonal_leg(0, self.letter_height / 2, -1)
        pop()

    def S(self, x, y):
        push()
        translate(x, y)
        # no_fill()
        self.s_curve(0, 0)
        pop()

    def T(self, x, y):
        push()
        translate(x, y)
        self.full_stem(self.letter_width / 2, 0)
        self.cross_bar(0, 0)
        pop()

    def U(self, x, y):
        push()
        translate(x, y)
        self.u_curve(0, 0)
        pop()

    def V(self, x, y):
        push()
        translate(x, y)
        self.diagonal_to_middle(0, 0, 1)
        self.diagonal_to_middle(0, 0, -1)
        pop()

    def W(self, x, y):
        push()
        translate(x, y)
        self.diagonal_to_quarter(0, 0, 1)
        self.diagonal_to_quarter(0, 0, -1)

        self.diagonal_to_quarter(self.letter_width / 2, 0, 1)
        self.diagonal_to_quarter(self.letter_width / 2, 0, -1)
        pop()

    def X(self, x, y):
        push()
        translate(x, y)
        self.diagonal_to_end(0, 0, -1)
        self.diagonal_to_end(0, 0, 1)
        pop()

    def Y(self, x, y):
        push()
        translate(x, y)
        self.half_stem(self.letter_width / 2, self.letter_height / 2)
        self.half_diagonal_arm(0, 0, 1)
        self.half_diagonal_arm(0, 0, -1)
        pop()

    def Z(self, x, y):
        push()
        translate(x, y)
        self.diagonal_to_end(0, 0, 1)
        self.cross_bar(0, 0)
        self.cross_bar(0, self.letter_height)
        pop()

    #
    # Component Definitions
    #

    def s_curve(self, x1, y1):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            curve(self.letter_width, -self.letter_height * 1.5, 0, self.letter_height * 0.25, self.letter_width, self.letter_height * 0.75, self.letter_width - self.letter_width, self.letter_height * 0.75 + self.letter_height * 1.5)
            # p5 draws an arc from PI to 0 clockwise through the top, which is PI to TWO_PI here
            arc(self.letter_width / 2, self.letter_height * 0.25, self.letter_width, self.letter_height / 2, PI, TWO_PI)
            arc(self.letter_width / 2, self.letter_height * 0.75, self.letter_width, self.letter_height / 2, 0, PI)

        def dynamic():
            self.arc_from_to_in_steps(self.letter_width / 2, self.letter_height * 0.25, self.letter_width / 2, self.letter_width / 2, -PI, 0, self.ani_steps)
            self.arc_from_to_in_steps(self.letter_width / 2, self.letter_height * 0.75, self.letter_width / 2, self.letter_width / 2, PI, 0, self.ani_steps)
            self.curve_from_to_in_steps(self.letter_width, -self.letter_height * 1.5, 0, self.letter_height * 0.25, self.letter_width, self.letter_height * 0.75, self.letter_width - self.letter_width, self.letter_height * 0.75 + self.letter_height * 1.5, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def u_curve(self, x1, y1):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            arc(self.letter_width / 2, 0, self.letter_width, self.letter_height * 2, 0, PI)

        def dynamic():
            self.arc_from_to_in_steps(self.letter_width / 2, 0, self.letter_width / 2, self.letter_height, 0, PI, self.ani_steps + 10)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def j_curve(self, x1, y1):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            bezier(self.letter_width, 0, self.letter_width + 10, self.letter_height * 1.5, 0, self.letter_height, 0, self.letter_height * 0.75)

        def dynamic():
            self.bezier_from_to_in_steps(self.letter_width, 0, self.letter_width + 10, self.letter_height * 1.5, 0, self.letter_height, 0, self.letter_height * 0.75, self.ani_steps + 10)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def letter_o(self, x1, y1):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            ellipse(self.letter_width / 2, self.letter_height / 2, self.letter_width, self.letter_height)

        def dynamic():
            self.arc_from_to_in_steps(self.letter_width / 2, self.letter_height / 2, self.letter_width / 2, self.letter_height / 2, PI, -PI, self.ani_steps + 10)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def half_diagonal_arm(self, x1, y1, direction):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            if direction == 1:
                line(self.letter_width / 2, self.letter_height / 2, self.letter_width, 0)
            if direction == -1:
                line(0, 0, self.letter_width / 2, self.letter_height / 2)

        def dynamic():
            if direction == 1:
                self.line_from_to_in_steps(self.letter_width / 2, self.letter_height / 2, self.letter_width, 0, self.ani_steps)
            if direction == -1:
                self.line_from_to_in_steps(0, 0, self.letter_width / 2, self.letter_height / 2, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def diagonal_to_end(self, x1, y1, direction):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            if direction == 1:
                line(self.letter_width, 0, 0, self.letter_height)
            if direction == -1:
                line(0, 0, self.letter_width, self.letter_height)

        def dynamic():
            if direction == 1:
                self.line_from_to_in_steps(self.letter_width, 0, 0, self.letter_height, self.ani_steps)
            if direction == -1:
                self.line_from_to_in_steps(0, 0, self.letter_width, self.letter_height, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def diagonal_to_quarter(self, x1, y1, direction):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            if direction == 1:
                line(0, 0, self.letter_width / 4, self.letter_height)
            if direction == -1:
                line(self.letter_width / 4, self.letter_height, self.letter_width / 2, 0)

        def dynamic():
            if direction == 1:
                self.line_from_to_in_steps(0, 0, self.letter_width / 4, self.letter_height, self.ani_steps)
            if direction == -1:
                self.line_from_to_in_steps(self.letter_width / 4, self.letter_height, self.letter_width / 2, 0, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def diagonal_to_middle(self, x1, y1, direction):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            if direction == 1:
                line(0, 0, self.letter_width / 2, self.letter_height)
            if direction == -1:
                line(self.letter_width / 2, self.letter_height, self.letter_width, 0)

        def dynamic():
            if direction == 1:
                self.line_from_to_in_steps(0, 0, self.letter_width / 2, self.letter_height, self.ani_steps)
            if direction == -1:
                self.line_from_to_in_steps(self.letter_width / 2, self.letter_height, self.letter_width, 0, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def half_diagonal_leg(self, x1, y1, direction):
        push()
        translate(x1, y1)
        endpoint = 0
        if direction == 1:
            endpoint = -self.letter_height / 2
        if direction == -1:
            endpoint = self.letter_height / 2

        def static():
            no_fill()
            line(0, 0, self.letter_height / 2, endpoint)

        def dynamic():
            self.line_from_to_in_steps(0, 0, self.letter_height / 2, endpoint, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def full_stem(self, x1, y1):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            line(0, 0, 0, self.letter_height)

        def dynamic():
            self.line_from_to_in_steps(0, 0, 0, self.letter_height, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def half_stem(self, x1, y1):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            line(0, 0, 0, self.letter_height / 2)

        def dynamic():
            self.line_from_to_in_steps(0, 0, 0, self.letter_height / 2, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def half_cross_bar(self, x1, y1):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            line(0, 0, self.letter_width / 2, 0)

        def dynamic():
            self.line_from_to_in_steps(0, 0, self.letter_width / 2, 0, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def half_bowl(self, x1, y1, direction):
        c_point = self.letter_width * 8
        no_fill()
        push()
        if direction == 1:
            translate(x1 + self.letter_width, y1)
        if direction == -1:
            translate(x1, y1)

        def static():
            no_fill()
            curve(c_point * direction, 0, 0, 0, 0, self.letter_height / 2, c_point * direction, self.letter_height / 2)

        def dynamic():
            self.curve_from_to_in_steps(c_point * direction, 0, 0, 0, 0, self.letter_height / 2, c_point * direction, self.letter_height / 2, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def full_bowl(self, x1, y1, direction):
        c_point = self.letter_width * 8
        no_fill()
        push()
        if direction == 1:
            translate(x1 + self.letter_width, y1)
        if direction == -1:
            translate(x1, y1)

        def static():
            no_fill()
            curve(c_point * direction, 0, 0, 0, 0, self.letter_height, c_point * direction, self.letter_height)

        def dynamic():
            self.curve_from_to_in_steps(c_point * direction, 0, 0, 0, 0, self.letter_height, c_point * direction, self.letter_height, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    def cross_bar(self, x1, y1):
        push()
        translate(x1, y1)

        def static():
            no_fill()
            line(0, 0, self.letter_width, 0)

        def dynamic():
            self.line_from_to_in_steps(0, 0, self.letter_width, 0, self.ani_steps)

        if self.draw_mode == 1:
            static()
        if self.draw_mode == 2:
            dynamic()
        if self.draw_mode == 3:
            static()
            dynamic()
        pop()

    #
    # Animation functions
    #

    def line_from_to_in_steps(self, x1, y1, x2, y2, step_count):
        ani_index = frame_count % (step_count + 1)
        ratio = ani_index / step_count
        pos_x = lerp(x1, x2, ratio)
        pos_y = lerp(y1, y2, ratio)
        fill(65 / 255, 105 / 255, 185 / 255)
        rect_mode(CENTER)
        if self.style == 1:
            rect(pos_x, pos_y, 10, 10)
        elif self.style == 2:
            ellipse(pos_x, pos_y, 10, 10)
        else:
            ellipse(pos_x, pos_y, 10, 10)

    def curve_from_to_in_steps(self, a1, a2, b1, b2, c1, c2, d1, d2, step_count):
        points = []
        for i in range(step_count + 1):
            t = i / step_count
            cx = curve_point(a1, b1, c1, d1, t)
            cy = curve_point(a2, b2, c2, d2, t)
            points.append([cx, cy])
        ani_index = frame_count % step_count
        ratio = ani_index / step_count
        pos_x = lerp(points[ani_index][0], points[ani_index + 1][0], ratio)
        pos_y = lerp(points[ani_index][1], points[ani_index + 1][1], ratio)
        fill(65 / 255, 105 / 255, 185 / 255)
        rect_mode(CENTER)
        if self.style == 1:
            rect(pos_x, pos_y, 10, 10)
        elif self.style == 2:
            ellipse(pos_x, pos_y, 10, 10)
        else:
            ellipse(pos_x, pos_y, 10, 10)

    def bezier_from_to_in_steps(self, a1, a2, b1, b2, c1, c2, d1, d2, step_count):
        points = []
        for i in range(step_count + 1):
            t = i / step_count
            cx = bezier_point(a1, b1, c1, d1, t)
            cy = bezier_point(a2, b2, c2, d2, t)
            points.append([cx, cy])
        ani_index = frame_count % step_count
        ratio = ani_index / step_count
        pos_x = lerp(points[ani_index][0], points[ani_index + 1][0], ratio)
        pos_y = lerp(points[ani_index][1], points[ani_index + 1][1], ratio)
        fill(65 / 255, 105 / 255, 185 / 255)
        rect_mode(CENTER)
        if self.style == 1:
            rect(pos_x, pos_y, 10, 10)
        elif self.style == 2:
            ellipse(pos_x, pos_y, 10, 10)
        else:
            ellipse(pos_x, pos_y, 10, 10)

    def arc_from_to_in_steps(self, x, y, radius_width, radius_height, a1, a2, step_count):
        ani_index = frame_count % (step_count + 1)
        ratio = ani_index / step_count
        angle = lerp(a1, a2, ratio)
        pos_x = x + cos(angle) * radius_width
        pos_y = y + sin(angle) * radius_height
        fill(65 / 255, 105 / 255, 185 / 255)
        rect_mode(CENTER)
        if self.style == 1:
            rect(pos_x, pos_y, 10, 10)
        elif self.style == 2:
            ellipse(pos_x, pos_y, 10, 10)
        else:
            ellipse(pos_x, pos_y, 10, 10)

# end AniLetters class


ani_letters = None


def setup():
    global ani_letters
    size(800, 800)
    stroke_weight(1)
    stroke_cap(ROUND)

    # adding your ani_letters object with
    # the letter width and height
    ani_letters = AniLetters(40, 100)

    # initialize with a message
    ani_letters.text_typed.append(ani_letters.add_text("TYPE"))
    ani_letters.text_typed.append(ani_letters.add_text("CODE"))


def draw():
    # no_loop()
    background(1.0, 1.0, 1.0, 30 / 255)  # GAP: background() with alpha does not fade the previous frame as in p5, it clears the canvas (no motion trails)

    # count how many lines of text there are
    ani_letters.get_line_count()
    # collect the letters and their x and y locations
    ani_letters.get_paths()
    # loop through the paths and draw them to screen
    ani_letters.render()


def key_pressed():
    if key_code == CONTROL:
        save_frame(timestamp() + ".png")

    # change draw mode
    if key_code == LEFT_ARROW:
        ani_letters.draw_mode -= 1
        if ani_letters.draw_mode < 1:
            ani_letters.draw_mode = 3
    if key_code == RIGHT_ARROW:
        ani_letters.draw_mode += 1
        if ani_letters.draw_mode > 3:
            ani_letters.draw_mode = 1
    # change the number of steps in the animation
    if key_code == DOWN:
        ani_letters.ani_steps -= 1
        if ani_letters.ani_steps < 4:
            ani_letters.ani_steps = 4
    if key_code == UP:
        ani_letters.ani_steps += 1

    # change between ellipses and rect
    if key == "1":
        ani_letters.style = 1
    if key == "2":
        ani_letters.style = 2

    # on return
    if key_code == ENTER:
        ani_letters.add_lines()
    # remove letters
    if key_code == DELETE or key_code == BACKSPACE:
        # ani_letters.text_typed.pop()
        # ani_letters.cursor_location["x"] -= ani_letters.letter_width + ani_letters.letter_padding
        ani_letters.remove_letters()


def key_typed():
    if key is not None and ord(key) >= 32:
        ani_letters.add_characters(key)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
