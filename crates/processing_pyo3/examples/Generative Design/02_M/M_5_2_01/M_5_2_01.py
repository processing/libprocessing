# M_5_2_01
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
# Ported from the Processing original to processing_py.

# press 'o' to select an input folder!
# console output only!
# take care of very big folders, loading will take up to several minutes.
#
# program takes a file directory (hierarchical tree) on harddisk
# as input and prints all filenames to the console.
#
# and shows also how to traverse a hierarchical tree in two
# different ways: Depth First Search + Breadth First Search
#
# KEYS
# o                  : select an input folder
import os

from processing_py import *


# This class code is based on code and ideas of chapter 7 in
# Visualizing Data, First Edition by Ben Fry. Copyright 2008 Ben Fry, 9780596514556.
class FileSystemItem:
    # ------ constructor ------
    def __init__(self, the_file):
        self.file = the_file
        self.children = []
        self.child_count = 0

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


# ------ default folder path ------
default_folder_path = os.path.expanduser("~") + "/Desktop"
# default_folder_path = "/Users/admin/Desktop"
# default_folder_path = "C:\\windows"

# ------ program logic ------
file_counter = 0


def setup():
    size(128, 128)
    set_input_folder(default_folder_path)


def draw():
    # keep the app running
    pass


# ------ folder selection dialog + init visualization ------
def set_input_folder(the_folder_path):
    # get files on harddisk
    print("\n" + str(the_folder_path))
    selected_folder = FileSystemItem(str(the_folder_path))
    selected_folder.print_depth_first()
    print("\n")
    selected_folder.print_breadth_first()


def key_released():
    if key == "o" or key == "O":
        select_folder("please select a folder", set_input_folder)  # GAP: select_folder(prompt, callback) folder chooser dialog is missing


run()
