# M_2_6_01_TOOL
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

# drawing tool that uses the special drawing method of connecting
# all points with every other.
#
# MOUSE
# left click          : set points
# shift + left click  : remove points
# right click + drag  : drag canvas
#
# KEYS
# backspace           : delete last point
# s                   : save png
# i                   : import points from a text file
# e                   : export points to a text file
#
# The controlP5 panel of the original is not ported; its parameters are the
# variables below, change their values to explore the tool. The panel's buttons
# called select_file() (load a background image) and reset().
from datetime import datetime

from processing_py import *

# ------ initial parameters and declarations ------

background_image_path = None
background_image = None
image_alpha = 30  # slider 0 to 100

eraser_radius = 20  # slider 1 to 50

# minimum distance to previously set point
min_distance = 10  # slider 0 to 30

zoom = 1  # slider 0.2 to 5

drawing = False
shift_down = False

point_count = 0
point_list = []

invert_background = False  # toggle
line_weight = 1  # slider 1 to 50
line_alpha = 50  # slider 0 to 100

connect_all_points = True  # toggle
connection_radius = 150  # slider 1 to 500
i1 = 0

min_hue_value = 0  # range slider 0 to 720 (low value)
max_hue_value = 100  # range slider 0 to 720 (high value)
saturation_value = 0  # slider 0 to 100
brightness_value = 0  # slider 0 to 100
invert_hue = False  # toggle

# ------ mouse interaction ------

dragging = False
offset_x = 0
offset_y = 0
click_x = 0
click_y = 0
click_offset_x = 0
click_offset_y = 0

# ------ image output ------

save_one_frame = False


def setup():
    size(800, 800)

    background(1.0)


def draw():
    global offset_x, offset_y, i1, point_count, save_one_frame
    # alpha values are in the range 0 to 100
    push_matrix()
    translate(width / 2, height / 2)
    scale(zoom)
    translate(-width / 2 + offset_x, -height / 2 + offset_y)

    bg_color = color(1.0)
    eraser_color = color(0.0)
    if invert_background:
        bg_color = color(0.0)
        eraser_color = color(1.0)

    if save_one_frame or i1 == 0:
        i1 = 0

    # canvas dragging
    if dragging:
        offset_x = click_offset_x + (mouse_x - click_x) / zoom
        offset_y = click_offset_y + (mouse_y - click_y) / zoom
        i1 = 0

    # set or delete points
    if drawing:
        if key_is_pressed and key_code == SHIFT:
            # shift pressed -> delete points
            for i in range(len(point_list) - 1, -1, -1):
                p = point_list[i]
                x = (mouse_x - width / 2) / zoom - offset_x + width / 2
                y = (mouse_y - height / 2) / zoom - offset_y + height / 2
                if dist(p.x, p.y, x, y) <= (eraser_radius / zoom):
                    del point_list[i]
            point_count = len(point_list)
            i1 = 0
        else:
            # set points
            x = (mouse_x - width / 2) / zoom - offset_x + width / 2
            y = (mouse_y - height / 2) / zoom - offset_y + height / 2
            if point_count > 0:
                p = point_list[point_count - 1]
                if dist(x, y, p.x, p.y) > min_distance:
                    point_list.append(Vec3(x, y, 0))
            else:
                point_list.append(Vec3(x, y, 0))
            point_count = len(point_list)

    # ------ draw everything ------

    stroke_weight(line_weight)
    stroke(0.0, line_alpha / 100)
    stroke_cap(ROUND)
    no_fill()
    tint(1.0, image_alpha / 100)

    if not connect_all_points or shift_down or dragging:
        background(bg_color)
        if background_image is not None and not save_one_frame:
            image(background_image, 0, 0)

        # simple drawing method
        for i in range(1, point_count):
            p1 = point_list[i - 1]
            p2 = point_list[i]
            draw_line(p1, p2)
            i1 += 1
    else:
        # drawing method where all points are connected with each other
        # alpha depends on distance of the points

        # clear background if drawing of the lines will start from the beginning
        if i1 == 0:
            background(bg_color)
            if background_image is not None and not save_one_frame:
                image(background_image, 0, 0)

        # draw lines not all at once, just the next 100 milliseconds to keep performance
        draw_end_time = millis() + 100
        if save_one_frame:
            draw_end_time = 2147483647

        while i1 < point_count and millis() < draw_end_time:
            for i2 in range(i1):
                p1 = point_list[i1]
                p2 = point_list[i2]
                draw_line(p1, p2)
            i1 += 1

    if shift_down or dragging:
        stroke(0.0)
        for i in range(point_count):
            p = point_list[i]
            point(p.x, p.y)

    pop_matrix()

    # draw eraser
    if key_is_pressed and key_code == SHIFT:
        stroke_weight(1)
        stroke(eraser_color)
        no_fill()
        ellipse(mouse_x, mouse_y, eraser_radius * 2, eraser_radius * 2)

    # ------ image output ------

    if save_one_frame:
        save_frame(timestamp() + ".png")
        save_one_frame = False


def draw_line(p1, p2):
    d = dist(p1.x, p1.y, p2.x, p2.y)
    a = pow(1 / (d / connection_radius + 1), 6)

    if d <= connection_radius:
        if not invert_hue:
            h = remap(a, 0, 1, min_hue_value, max_hue_value) % 360
        else:
            h = remap(1 - a, 0, 1, min_hue_value, max_hue_value) % 360
        stroke(hsva(h, saturation_value / 100, brightness_value / 100, (a * line_alpha + (i1 % 2 * 2)) / 100))
        line(p1.x, p1.y, p2.x, p2.y)


def reset():
    global point_count, i1, offset_x, offset_y
    global image_alpha, eraser_radius, zoom, invert_background, line_weight, line_alpha
    global min_hue_value, max_hue_value, saturation_value, brightness_value, invert_hue
    global connection_radius, connect_all_points
    point_list.clear()
    point_count = 0
    i1 = 0
    offset_x = 0
    offset_y = 0

    # reset parameters
    image_alpha = 30.0
    eraser_radius = 20.0
    zoom = 1.0
    invert_background = False
    line_weight = 1.0
    line_alpha = 50.0
    min_hue_value = 0
    max_hue_value = 100
    saturation_value = 0.0
    brightness_value = 0.0
    invert_hue = False
    connection_radius = 150.0
    connect_all_points = True


def select_file():
    # opens file chooser
    select_input("Select an image file", file_selected)  # GAP: select_input(prompt, callback) file chooser dialog is missing


def file_selected(selection):
    global background_image_path, background_image
    if selection is not None:
        background_image_path = str(selection)
        print(background_image_path)
        background_image = load_image(background_image_path)


def select_file_load_points():
    # opens file chooser
    select_input("Select Text File with Point Information", load_point_path_selected)  # GAP: select_input(prompt, callback) file chooser dialog is missing


def load_point_path_selected(selection):
    global point_count, i1
    if selection is not None:
        load_points_path = str(selection)
        point_file = open(load_points_path)
        point_strings = point_file.read().splitlines()
        point_file.close()

        point_count = 0
        point_list.clear()
        for i in range(len(point_strings)):
            pt = point_strings[i].split(" ")
            if len(pt) == 2:
                point_list.append(Vec3(float(pt[0]), float(pt[1]), 1))
                point_count += 1
            elif len(pt) == 3:
                point_list.append(Vec3(float(pt[0]), float(pt[1]), float(pt[2])))
                point_count += 1
        i1 = 0


def select_file_save_points():
    # opens file chooser
    select_output("Save Points as Text File", save_point_path_selected)  # GAP: select_output(prompt, callback) file chooser dialog is missing


def save_point_path_selected(selection):
    if selection is not None:
        save_points_path = str(selection)
        if not save_points_path.endswith(".txt"):
            save_points_path += ".txt"

        point_strings = []
        for i in range(point_count):
            p = point_list[i]
            if p.z > 0:
                point_strings.append(str(p.x) + " " + str(p.y) + " " + str(p.z))
            else:
                point_strings.append(str(p.x) + " " + str(p.y) + " " + str(1))
        point_file = open(save_points_path, "w")
        point_file.write("\n".join(point_strings))
        point_file.close()


def key_pressed():
    global save_one_frame, point_count, i1, shift_down
    if key == "s" or key == "S":
        save_one_frame = True

    if key_code == BACKSPACE:
        point_count -= 1
        point_count = max(point_count, 0)
        if len(point_list) > 0:
            del point_list[point_count]
        i1 = 0

    if key_code == SHIFT:
        shift_down = True

    if key == "e" or key == "E":
        select_file_save_points()
    if key == "i" or key == "I":
        select_file_load_points()


def key_released():
    global shift_down, i1
    if shift_down:
        shift_down = False
        i1 = 0


def mouse_pressed():
    global drawing, dragging, click_x, click_y, click_offset_x, click_offset_y
    if mouse_button == LEFT:
        drawing = True
    if mouse_button == RIGHT:
        dragging = True
        click_x = mouse_x
        click_y = mouse_y
        click_offset_x = offset_x
        click_offset_y = offset_y


def mouse_released():
    global drawing, dragging, i1
    drawing = False
    if dragging:
        dragging = False
        i1 = 0


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
