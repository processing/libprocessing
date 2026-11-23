# P_3_2_5_03
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

# Animated type exploring various drawing modes
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


class AnimatedType:
    def __init__(self):
        self.text_typed = []
        self.paths = []
        self.individual_paths = []
        self.ranges = []
        self.line_count = 0
        self.letter_coordinates = []
        self.coordinates = []
        self.point_density = 2
        self.start_x = 0
        self.colors = [
            color(65 / 255, 105 / 255, 185 / 255),
            color(245 / 255, 95 / 255, 80 / 255),
            color(15 / 255, 233 / 255, 118 / 255),
            color(233 / 255, 15 / 255, 130 / 255),
            color(118 / 255, 15 / 255, 233 / 255),
            color(15 / 255, 233 / 255, 118 / 255),
        ]
        self.angle = 0

        self.draw_mode = 8
        self.style = 1

    #
    # Data Handling Methods
    #

    # set the line_count to the number of "lines" or text object in the text_typed list
    def get_line_count(self):
        if len(self.text_typed) > 0:
            self.line_count = len(self.text_typed) - 1
        else:
            self.line_count = 0

    # create a text object to hold each line of text
    # usage: self.text_typed.append(self.add_text("hello"))
    def add_text(self, _text):
        text_object = {"counter": 0, "text": _text}
        return text_object

    # get the path objects for each line typed
    def get_paths(self):
        # clear the paths each loop
        self.paths = []

        text_font(font)
        text_size(font_size)

        # go though each of the text objects
        for line_num in range(len(self.text_typed)):
            txt = self.text_typed[line_num]
            if len(txt["text"]) > 0:
                # get the font outline, resampled with points every point_density pixels
                path = text_to_points(txt["text"], 0, 0, 1 / self.point_density)

                # structure the relevant path data
                path_data = {
                    "data": path,
                    "line_number": line_num,
                    "len": len(path),
                    "breaks": floor(len(path) / len(txt["text"])),
                    "ranges": [],
                }

                # get the start point of each letter
                if path_data["breaks"] > 0:
                    for i in range(0, path_data["len"] - 1, path_data["breaks"]):
                        path_data["ranges"].append(floor(i))

                self.paths.append(path_data)

    def get_individual_paths(self):
        self.individual_paths = []

        text_font(font)
        text_size(font_size)

        # go though each of the text objects
        for line_num in range(len(self.text_typed)):
            txt = self.text_typed[line_num]
            if len(txt["text"]) > 0:
                for d in txt["text"]:
                    # get the font outline, resampled with points every point_density pixels
                    path = text_to_points(d, 0, 0, 1 / self.point_density)

                    # structure the relevant path data
                    path_data = {
                        "data": path,
                        "line_number": line_num,
                        "len": len(path),
                        "bbox": get_bounding_box(path),
                        "dist_x": 0,
                        "start_x": 0,
                    }

                    # print(path_data["bbox"]["x1"])
                    path_data["dist_x"] = ceil(dist(path_data["bbox"]["x1"], 0, path_data["bbox"]["x2"], 0))

                    self.individual_paths.append(path_data)

        # set the start_x to zero
        self.start_x = 0
        for i in range(len(self.individual_paths) - 1):
            # if the linenumbers are the same
            if self.individual_paths[i]["line_number"] == self.individual_paths[i + 1]["line_number"]:
                # then add to the start_x and assign it to the individualpath start_x
                self.individual_paths[i]["start_x"] = self.start_x
                self.start_x += self.individual_paths[i]["dist_x"] + 15
            else:
                self.individual_paths[i]["start_x"] = self.start_x
                self.start_x = 0
            # when reaching the end
            if i == len(self.individual_paths) - 2:
                self.individual_paths[i + 1]["start_x"] = self.start_x

    # get all the coordinates
    def get_coordinates(self):
        # clear the coordinates each loop
        self.coordinates = []

        # for each of the letters
        for path in self.paths:
            for coord in path["data"]:
                y_offset = path["line_number"] * font_size
                self.coordinates.append([coord[0], coord[1] + y_offset])

    #
    # keyboard interaction Methods
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
        self.text_typed[self.line_count]["text"] += _key

    #
    # Rendering Methods
    #

    # show all the points with random color
    def random_strokes(self):
        for coords in self.coordinates:
            stroke(random(1.0), random(1.0), random(1.0))
            ellipse(coords[0], coords[1], 5, 5)

    # follow the mouse with extruded lines
    def lines2mouse(self):
        stroke(self.colors[0])
        for coords in self.coordinates:
            stroke_weight(1)
            line(coords[0] + remap(mouse_x, 0, width, -100, 100), coords[1] + remap(mouse_y, 0, height, -100, 100), coords[0], coords[1])

    # animate the points
    def animated_points(self, _shape):
        for idx in range(len(self.paths)):
            path = self.paths[idx]
            fill(self.colors[path["line_number"]])
            stroke(self.colors[path["line_number"]])
            for d in path["ranges"]:
                cmd_index = self.text_typed[idx]["counter"] + d

                if cmd_index < len(path["data"]):
                    cmd = path["data"][cmd_index]
                    if self.text_typed[idx]["counter"] < path["breaks"]:
                        y_offset = path["line_number"] * font_size
                        if _shape == "ellipse":
                            ellipse(cmd[0], cmd[1] + y_offset, font_size * 0.10, font_size * 0.10)
                        elif _shape == "rect":
                            rect(cmd[0], cmd[1] + y_offset, font_size * 0.10, font_size * 0.10)

                        self.text_typed[idx]["counter"] += 1
                    else:
                        self.text_typed[idx]["counter"] = 0

    # radial lines
    def radial_lines(self):
        stroke(self.colors[0])
        for coords in self.coordinates:
            stroke_weight(1)
            for i in range(0, 360, 60):
                angle = radians(i)
                radius_distance_x = remap(mouse_x, 0, width, 0, random(20))
                radius_distance_y = remap(mouse_y, 0, width, 0, random(20))

                pt_x = cos(angle) * radius_distance_x + coords[0]
                pt_y = sin(angle) * radius_distance_y + coords[1]
                line(pt_x, pt_y, coords[0], coords[1])

    # orbiting points
    def orbiting_points(self, _type):
        stroke(self.colors[0])
        rect_size = font_size * 0.05
        shift_x1 = mouse_x / 100 * random(-1, 1)
        shift_y1 = mouse_y / 100 * random(-1, 1)
        shift_x2 = mouse_x / 100 * random(-1, 1)
        shift_y2 = mouse_y / 100 * random(-1, 1)
        shift_x3 = mouse_x / 100 * random(-1, 1)
        shift_y3 = mouse_y / 100 * random(-1, 1)
        shift_x4 = mouse_x / 100 * random(-1, 1)
        shift_y4 = mouse_y / 100 * random(-1, 1)

        for idx in range(len(self.coordinates)):
            coords = self.coordinates[idx]

            if idx % 2 == 0:
                angle = radians(frame_count % 360)
            else:
                angle = radians(-(frame_count % 360))

            if _type == "points":
                push()
                translate(coords[0], coords[1])
                rotate(angle)
                point(0 + shift_x1, 0 + shift_y1)
                point(0 + rect_size + shift_x2, 0 + shift_y2)
                pop()

    # make a triangle blob from the points
    def triangle_blob(self):
        fill(self.colors[1])
        stroke(self.colors[1])

        begin_shape(TRIANGLE_STRIP)
        for coords in self.coordinates:
            vertex(coords[0], coords[1])
        end_shape()

    # wobbly_shapes
    def wobbly_shapes(self, _type):
        for idx in range(len(self.individual_paths)):
            path = self.individual_paths[idx]
            stroke(self.colors[path["line_number"]])
            fill(self.colors[path["line_number"]])

            self.angle += 0.01
            if idx % 2 == 1:
                shifter = sin(self.angle) * 0.05
            else:
                shifter = sin(self.angle) * -0.05

            y_offset = path["line_number"] * font_size
            x_offset = path["start_x"]

            push()
            translate(x_offset, y_offset)
            rotate(shifter)

            # choose a begin_shape mode
            if _type == "TRIANGLES":
                begin_shape(TRIANGLES)
            elif _type == "TRIANGLE_STRIP":
                begin_shape(TRIANGLE_STRIP)
            elif _type == "LINES":
                begin_shape(LINES)
            elif _type == "TRIANGLE_FAN":
                begin_shape(TRIANGLE_FAN)
            else:
                begin_shape(TRIANGLES)

            # add all those vertices to the shape
            for d in path["data"]:
                vertex(d[0], d[1])

            end_shape()
            pop()

    # outward lines following mouse
    def outward_lines(self):
        for idx in range(len(self.individual_paths)):
            path = self.individual_paths[idx]
            stroke(self.colors[path["line_number"]])
            stroke_weight(0.5)
            fill(self.colors[path["line_number"]])
            y_offset = path["line_number"] * font_size
            x_offset = path["start_x"]

            push()
            translate(x_offset, y_offset)
            c_x = (path["bbox"]["x1"] + path["bbox"]["x2"]) / 2 + remap(mouse_x, 0, width, -50, 50)
            c_y = (path["bbox"]["y1"] + path["bbox"]["y2"]) / 2 + remap(mouse_y, 0, height, -50, 50)
            # add all those vertices to the shape
            for d in path["data"]:
                line(c_x, c_y, d[0], d[1])
            pop()


# end AnimatedType class


# the smallest box around all the points of an outline
def get_bounding_box(points):
    bbox = {"x1": 0, "y1": 0, "x2": 0, "y2": 0}
    for i in range(len(points)):
        x = points[i][0]
        y = points[i][1]
        if i == 0:
            bbox = {"x1": x, "y1": y, "x2": x, "y2": y}
        bbox["x1"] = min(bbox["x1"], x)
        bbox["y1"] = min(bbox["y1"], y)
        bbox["x2"] = max(bbox["x2"], x)
        bbox["y2"] = max(bbox["y2"], y)
    return bbox


font = None
font_size = 120
# declare your animated_text variable
my_animated_text = None


def setup():
    global font, my_animated_text
    full_screen()
    no_cursor()
    image_mode(CENTER)
    rect_mode(CENTER)

    # initialize the AnimatedType object
    my_animated_text = AnimatedType()
    # add some text in
    my_animated_text.text_typed.append(my_animated_text.add_text("TYPE!"))
    my_animated_text.text_typed.append(my_animated_text.add_text("CODE!"))

    # read in the font
    font = load_font("data/FreeSans.otf")

    # frame_rate(1)


def draw():
    # no_loop()
    background(1.0, 1.0, 1.0, 20 / 255)  # GAP: background() with alpha does not fade the previous frame as in p5, it clears the canvas (no motion trails)
    # background(1.0, 1.0, 1.0)

    # margin border
    translate(20, 150)
    fill(0.0)

    my_animated_text.get_line_count()
    my_animated_text.get_paths()
    my_animated_text.get_individual_paths()
    my_animated_text.get_coordinates()

    # draw methods
    if my_animated_text.draw_mode == 1:
        my_animated_text.animated_points("ellipse")
    if my_animated_text.draw_mode == 2:
        my_animated_text.animated_points("rect")
    if my_animated_text.draw_mode == 3:
        my_animated_text.lines2mouse()
    if my_animated_text.draw_mode == 4:
        my_animated_text.radial_lines()
    if my_animated_text.draw_mode == 5:
        my_animated_text.orbiting_points("points")
    if my_animated_text.draw_mode == 6:
        my_animated_text.wobbly_shapes("TRIANGLE_FAN")
    if my_animated_text.draw_mode == 7:
        my_animated_text.wobbly_shapes("TRIANGLES")
    if my_animated_text.draw_mode == 8:
        my_animated_text.outward_lines()


# key controls
def key_pressed():
    if key_code == CONTROL:
        save_frame(timestamp() + ".png")

    if key_code == DELETE or key_code == BACKSPACE:
        my_animated_text.remove_letters()

    if key_code == ENTER:
        my_animated_text.add_lines()

    if key_code == LEFT_ARROW:
        my_animated_text.draw_mode -= 1
        if my_animated_text.draw_mode < 1:
            my_animated_text.draw_mode = 8
    if key_code == RIGHT_ARROW:
        my_animated_text.draw_mode += 1
        if my_animated_text.draw_mode > 8:
            my_animated_text.draw_mode = 1

    if key_code == UP:
        my_animated_text.point_density -= 1
        if my_animated_text.point_density < 2:
            my_animated_text.point_density = 2
    if key_code == DOWN:
        my_animated_text.point_density += 1


def key_typed():
    if key is not None and ord(key) >= 32:
        my_animated_text.add_characters(key)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
