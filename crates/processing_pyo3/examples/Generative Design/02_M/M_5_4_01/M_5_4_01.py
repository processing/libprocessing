# M_5_4_01
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
# CREDITS
# part of the FileSystemItem class is based on code from Visualizing Data, First Edition
# by Ben Fry. Copyright 2008 Ben Fry, 9780596514556.
#
# calcEqualAreaRadius function was done by Prof. Franklin Hernandez-Castro
#
# Ported from the Processing original to processing_py.

# press 'o' to select an input folder!
# take care of very big folders, loading will take up to several minutes.
#
# program takes a file directory (hierarchical tree) as input.
# also all files and folders are displayed with sunburst technique
# and in addition all relations of the files are visualized with lines.
#
# KEYS
# o                          : select an input folder
# 1                          : mappingMode -> last modified
# 2                          : mappingMode -> file size
# 3                          : mappingMode -> local level file size
# b                          : toogle, use bezier or straight lines
# s                          : save png
import os
import time
from datetime import datetime

from processing_py import *


# This class code is based on code and ideas of chapter 7 in
# Visualizing Data, First Edition by Ben Fry. Copyright 2008 Ben Fry, 9780596514556.
class FileSystemItem:
    # ------ constructor ------
    def __init__(self, the_file):
        self.file = the_file
        self.children = []
        self.child_count = 0
        self.folder_min_filesize = 0.0
        self.folder_max_filesize = 0.0

        if os.path.isdir(self.file):
            try:
                contents = os.listdir(self.file)
            except OSError:
                contents = None
            if contents is not None:
                # Sort the file names in case insensitive order
                contents.sort()

                for i in range(len(contents)):
                    # skip the . and .. directory entries on Unix systems
                    if contents[i] == "." or contents[i] == ".." or contents[i][0:1] == ".":
                        continue
                    child_file = os.path.join(self.file, contents[i])
                    # skip any file that appears to be a symbolic link
                    abs_path = os.path.abspath(child_file)
                    can_path = os.path.realpath(child_file)
                    if abs_path != can_path:
                        continue
                    child = FileSystemItem(child_file)
                    self.children.append(child)
                    self.child_count += 1

                    self.folder_min_filesize = min(child.get_file_size(), self.folder_min_filesize)
                    self.folder_max_filesize = max(child.get_file_size(), self.folder_max_filesize)

                # remember the biggest and smallest filesite on each depth
                for i in range(self.child_count):
                    self.children[i].folder_min_filesize = self.folder_min_filesize
                    self.children[i].folder_max_filesize = self.folder_max_filesize

    # ------ get file size function ------
    # how many MEGABYTE has my FileSystemItem
    def get_file_size(self):
        megabyte = 1048576
        return self.get_file_size_of(self) / megabyte

    def get_file_size_of(self, hd_item):
        try:
            if os.path.isfile(hd_item.file):
                return os.path.getsize(hd_item.file)
            hd_items = hd_item.children
            total_size = 0
            for i in range(len(hd_items)):
                total_size += self.get_file_size_of(hd_items[i])
            return total_size
        except OSError:
            return 0

    # ------ compare last file modification with current date function ------
    def get_not_modified_since(self, the_file):
        # get the date in milliseconds
        # current file
        try:
            milis1 = os.path.getmtime(the_file) * 1000
        except OSError:
            milis1 = 0
        # now, global var
        milis2 = now * 1000
        # calculate difference in milliseconds
        diff = milis2 - milis1
        # calculate difference in days
        diff_days = diff / (24 * 60 * 60 * 1000)
        return int(diff_days)

    # ------ print and debug functions ------
    # Depth First Search
    def print_depth_first(self):
        global file_counter
        print("printDepthFirst")
        # global file_counter
        file_counter = 0
        self.print_depth_first_item(0, -1)
        print(str(file_counter) + " files")

    def print_depth_first_item(self, depth, index_to_parent):
        global file_counter
        # print four spaces for each level of depth + debug println
        for i in range(depth):
            print("    ", end="")
        print(str(file_counter) + " " + str(index_to_parent) + "<-->" + str(file_counter) + " (" + str(depth) + ") " + os.path.basename(self.file))

        index_to_parent = file_counter
        file_counter += 1
        # now handle the children, if any
        for i in range(self.child_count):
            self.children[i].print_depth_first_item(depth + 1, index_to_parent)

    # Breadth First Search
    def print_breadth_first(self):
        print("printBreadthFirst")

        # queues for pushing and saving all elements in "breadth first search" style
        items = []
        depths = []
        indices_parent = []

        # add first elements and startingpoint
        items.append(self)
        depths.append(0)
        indices_parent.append(-1)

        # tmp vars for running in while loop
        index = 0
        item_count = 1

        while item_count > index:
            item = items[index]
            depth = depths[index]
            index_to_parent = indices_parent[index]

            # print four spaces for each level of depth + debug println
            for i in range(depth):
                print("    ", end="")
            print(str(index) + " " + str(index_to_parent) + "<-->" + str(index) + " (" + str(depth) + ") " + os.path.basename(item.file))

            # is current node a directory?
            # yes -> push all children to the end of the items
            if os.path.isdir(item.file):
                for i in range(item.child_count):
                    items.append(item.children[i])
                    depths.append(depth + 1)
                    indices_parent.append(index)
                item_count += item.child_count
            index += 1
        print(str(index) + " files")

    # ------ init viz items function ------
    # Breadth First
    def create_sunburst_items(self):
        print("createSunburstItems -> ", end="")
        megabytes = self.get_file_size()
        angle_per_megabyte = TWO_PI / megabytes

        # temp array for pushing and saving all elements in "breadth first search" style
        items = []
        depths = []
        indices_parent = []
        sunburst_items = []
        angles = []

        # add first elements and startingpoint
        items.append(self)
        depths.append(0)
        indices_parent.append(-1)
        angles.append(0.0)

        # tmp vars for running in while loop
        index = 0
        angle_offset = 0
        old_angle = 0

        while len(items) > index:
            item = items[index]
            depth = depths[index]
            index_to_parent = indices_parent[index]
            angle = angles[index]

            # if there is an angle change (= entering a new directory) reset angleOffset
            if old_angle != angle:
                angle_offset = 0.0

            # is current node a directory?
            # yes -> push all children to the end of the items
            if os.path.isdir(item.file):
                for ii in range(item.child_count):
                    items.append(item.children[ii])
                    depths.append(depth + 1)
                    indices_parent.append(index)
                    angles.append(angle + angle_offset)

            sunburst_items.append(SunburstItem(index, index_to_parent, item.child_count, depth,
                                               item.get_file_size(), self.get_not_modified_since(item.file),
                                               item.file, (angle + angle_offset) % TWO_PI, item.get_file_size() * angle_per_megabyte,
                                               item.folder_min_filesize, item.folder_max_filesize))

            angle_offset += item.get_file_size() * angle_per_megabyte
            index += 1
            old_angle = angle

        print(str(index) + " SunburstItems")

        return sunburst_items


class SunburstItem:
    # ------ constructor ------
    def __init__(self, the_index, the_index_to_parent, the_child_count, the_depth, the_file_size, the_last_modified, the_file, the_angle, the_extension, the_folder_min_filesize, the_folder_max_filesize):
        # relations
        self.depth = the_depth
        self.index = the_index
        self.index_to_parent = the_index_to_parent
        self.child_count = the_child_count

        # meta file informations
        self.file = the_file
        self.name = os.path.basename(the_file)
        self.is_dir = os.path.isdir(the_file)
        self.last_modified = the_last_modified  # age of the file in days
        self.file_size = the_file_size
        self.folder_min_filesize = the_folder_min_filesize  # biggest an smallest file in the same folder
        self.folder_max_filesize = the_folder_max_filesize

        # arc and lines drawing vars
        self.col = color(0.0)
        self.line_weight = 0
        self.radius = 0
        self.depth_weight = 0  # stroke weight of the arc
        self.x = 0
        self.y = 0
        self.arc_length = 0
        # bezier controlpoints
        self.c1_x = 0
        self.c1_y = 0
        self.c2_x = 0
        self.c2_y = 0

        # sunburst angles and extension
        self.angle_start = the_angle
        self.extension = the_extension
        self.angle_center = the_angle + the_extension / 2
        self.angle_end = the_angle + the_extension

    # ------ update function, called only when the input files are changed ------
    def update(self, the_mapping_mode):
        if self.index_to_parent > -1:
            self.radius = calc_equal_area_radius(self.depth, depth_max)
            self.depth_weight = calc_equal_area_radius(self.depth + 1, depth_max) - self.radius
            self.x = cos(self.angle_center) * self.radius
            self.y = sin(self.angle_center) * self.radius

            # chord
            start_x = cos(self.angle_start) * self.radius
            start_y = sin(self.angle_start) * self.radius
            end_x = cos(self.angle_end) * self.radius
            end_y = sin(self.angle_end) * self.radius
            self.arc_length = dist(start_x, start_y, end_x, end_y)

            # color mapings
            percent = 0
            if the_mapping_mode == 1:
                percent = norm(self.last_modified, last_modified_oldest, last_modified_youngest)
            elif the_mapping_mode == 2:
                percent = norm(self.file_size, file_size_min, file_size_max)
            elif the_mapping_mode == 3:
                percent = norm(self.file_size, self.folder_min_filesize, self.folder_max_filesize)

            # colors for files and folders
            if self.is_dir:
                bright = lerp(folder_brightness_start, folder_brightness_end, percent)
                self.col = hsva(0, 0, bright / 100)
            else:
                from_color = hsva(hue_start, saturation_start / 100, brightness_start / 100)
                to_color = hsva(hue_end, saturation_end / 100, brightness_end / 100)
                self.col = lerp_color(from_color, to_color, percent)

            # calc stroke weight for relations line
            self.line_weight = remap(self.depth, depth_max, 1, stroke_weight_start, stroke_weight_end)
            if self.arc_length < self.line_weight:
                self.line_weight = self.arc_length * 0.93

            # calc bezier controlpoints
            self.c1_x = cos(self.angle_center)
            self.c1_x *= calc_equal_area_radius(self.depth - 1, depth_max)

            self.c1_y = sin(self.angle_center)
            self.c1_y *= calc_equal_area_radius(self.depth - 1, depth_max)

            self.c2_x = cos(sunburst[self.index_to_parent].angle_center)
            self.c2_x *= calc_equal_area_radius(self.depth, depth_max)

            self.c2_y = sin(sunburst[self.index_to_parent].angle_center)
            self.c2_y *= calc_equal_area_radius(self.depth, depth_max)

    # ------ draw functions ------
    def draw_arc(self, the_folder_scale, the_file_scale):
        if self.depth > 0:
            if self.is_dir:
                stroke_weight(self.depth_weight * the_folder_scale)
                arc_radius = self.radius + self.depth_weight * the_folder_scale / 2
            else:
                stroke_weight(self.depth_weight * the_file_scale)
                arc_radius = self.radius + self.depth_weight * the_file_scale / 2
            stroke(self.col)
            arc(0, 0, arc_radius, arc_radius, self.angle_start, self.angle_end)

    def draw_dot(self):
        if self.depth > 0:
            diameter = dot_size
            if self.arc_length < diameter:
                diameter = self.arc_length * 0.95
            if self.depth == 0:
                diameter = 3
            fill(hsva(0, 0, dot_brightness / 100))
            no_stroke()
            ellipse(self.x, self.y, diameter, diameter)
            no_fill()

    def draw_relation_line(self):
        if self.depth > 0:
            stroke(self.col)
            stroke_weight(self.line_weight)
            line(self.x, self.y, sunburst[self.index_to_parent].x, sunburst[self.index_to_parent].y)

    def draw_relation_bezier(self):
        if self.depth > 1:
            stroke(self.col)
            stroke_weight(self.line_weight)
            bezier(self.x, self.y, self.c1_x, self.c1_y, self.c2_x, self.c2_y, sunburst[self.index_to_parent].x, sunburst[self.index_to_parent].y)
        # else:
        #     self.draw_relation_line()


# ------ default folder path ------
default_folder_path = os.path.expanduser("~") + "/Desktop"

# ------ control parameters ------
hue_start = 190
hue_end = 195
saturation_start = 90
saturation_end = 100
brightness_start = 25
brightness_end = 85
folder_brightness_start = 20
folder_brightness_end = 90
file_arc_scale = 0.2
folder_arc_scale = 0.1
stroke_weight_start = 1.0
stroke_weight_end = 2.5
dot_size = 1.5
dot_brightness = 1
use_bezier_line = False

mapping_mode = 1

# ------ program logic ------
sunburst = []
now = time.time()
depth_max = 0
last_modified_oldest = 0
last_modified_youngest = 0
file_size_min = 0
file_size_max = 0
child_count_min = 0
child_count_max = 0
file_counter = 0


def setup():
    size(1000, 800)
    set_input_folder(default_folder_path)


def draw():
    push_matrix()
    background(1.0)
    no_fill()
    ellipse_mode(RADIUS)
    stroke_cap(SQUARE)
    translate(width / 2, height / 2)

    # ------ draw the viz items ------
    # for i in range(len(sunburst)):
    #     sunburst[i].draw_arc(folder_arc_scale, file_arc_scale)

    for i in range(len(sunburst)):
        if use_bezier_line:
            sunburst[i].draw_relation_bezier()
        else:
            sunburst[i].draw_relation_line()

    for i in range(len(sunburst)):
        sunburst[i].draw_dot()

    pop_matrix()


# ------ folder selection dialog + init visualization ------
def set_input_folder(the_folder_path):
    global sunburst, depth_max, last_modified_oldest, last_modified_youngest
    global file_size_min, file_size_max, child_count_min, child_count_max
    # get files on harddisk
    print("\n" + str(the_folder_path))
    selected_folder = FileSystemItem(str(the_folder_path))
    # selected_folder.print_depth_first()
    # selected_folder.print_breadth_first()

    # init sunburst
    sunburst = selected_folder.create_sunburst_items()

    # mine sunburst -> get min and max values
    # reset the old values, without the root element
    depth_max = 0
    last_modified_oldest = 0
    last_modified_youngest = 0
    file_size_min = 0
    file_size_max = 0
    child_count_min = 0
    child_count_max = 0
    for i in range(1, len(sunburst)):
        depth_max = max(sunburst[i].depth, depth_max)
        last_modified_oldest = max(sunburst[i].last_modified, last_modified_oldest)
        last_modified_youngest = min(sunburst[i].last_modified, last_modified_youngest)
        file_size_min = min(sunburst[i].file_size, file_size_min)
        file_size_max = max(sunburst[i].file_size, file_size_max)
        child_count_min = min(sunburst[i].child_count, child_count_min)
        child_count_max = max(sunburst[i].child_count, child_count_max)

    # update vars
    for i in range(len(sunburst)):
        sunburst[i].update(mapping_mode)


# ------ returns radiuses to have equal areas in each depth ------
def calc_equal_area_radius(the_depth, the_depth_max):
    return sqrt(the_depth * pow(height // 2, 2) / (the_depth_max + 1))


# ------ returns radiuses in a linear way ------
def calc_area_radius(the_depth, the_depth_max):
    return remap(the_depth, 0, the_depth_max + 1, 0, height // 2)


# ------ interaction ------
def key_released():
    global mapping_mode, use_bezier_line
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == "o" or key == "O":
        select_folder("please select a folder", set_input_folder)  # GAP: select_folder(prompt, callback) folder chooser dialog is missing
    if key == "b" or key == "B":
        use_bezier_line = not use_bezier_line

    if key == "1":
        mapping_mode = 1
        window_title("last modified: old / young files, global")
    if key == "2":
        mapping_mode = 2
        window_title("file size: big / small files, global")
    if key == "3":
        mapping_mode = 3
        window_title("local folder file size: big / small files, each folder independent")
    if key == "1" or key == "2" or key == "3":
        for i in range(len(sunburst)):
            sunburst[i].update(mapping_mode)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
