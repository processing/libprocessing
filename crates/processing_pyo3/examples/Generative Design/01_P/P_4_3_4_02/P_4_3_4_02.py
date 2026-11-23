# P_4_3_4_02
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

# Pixel mapping, each pixel of a video input is translated into an emoji icon.
# The pixel RGB value is mapped with the nearest (average) RGB value of an emoji.
# Mapping via a [k-dimensional tree data structure](https://github.com/ubilabs/kd-tree-javascript).
#
# KEYS
# s                   : save png
# f                   : start/stop recording of frames
import json
from datetime import datetime

from processing_py import *


# k-d tree, ported from https://github.com/ubilabs/kd-tree-javascript (MIT License)
class KdNode:
    def __init__(self, obj, dimension, parent):
        self.obj = obj
        self.left = None
        self.right = None
        self.parent = parent
        self.dimension = dimension


class KdTree:
    def __init__(self, points, metric, dimensions):
        self.metric = metric
        self.dimensions = dimensions
        self.root = self.build_tree(points, 0, None)

    def build_tree(self, points, depth, parent):
        dim = depth % len(self.dimensions)

        if len(points) == 0:
            return None
        if len(points) == 1:
            return KdNode(points[0], dim, parent)

        key = self.dimensions[dim]
        points.sort(key=coordinate_getter(key))

        median = len(points) // 2
        node = KdNode(points[median], dim, parent)
        node.left = self.build_tree(points[:median], depth + 1, node)
        node.right = self.build_tree(points[median + 1 :], depth + 1, node)

        return node

    # returns a list of [point, distance] for the max_nodes nearest points
    def nearest(self, point, max_nodes):
        # the best nodes found so far as [node, distance], farthest first
        best_nodes = []

        if self.root is not None:
            self.nearest_search(self.root, point, max_nodes, best_nodes)

        result = []
        for i in range(min(max_nodes, len(best_nodes))):
            result.append([best_nodes[i][0].obj, best_nodes[i][1]])
        return result

    def nearest_search(self, node, point, max_nodes, best_nodes):
        dimension = self.dimensions[node.dimension]
        own_distance = self.metric(point, node.obj)

        linear_point = {}
        for i in range(len(self.dimensions)):
            if i == node.dimension:
                linear_point[self.dimensions[i]] = point[self.dimensions[i]]
            else:
                linear_point[self.dimensions[i]] = node.obj[self.dimensions[i]]

        linear_distance = self.metric(linear_point, node.obj)

        if node.right is None and node.left is None:
            if len(best_nodes) < max_nodes or own_distance < best_nodes[0][1]:
                self.save_node(node, own_distance, max_nodes, best_nodes)
            return

        if node.right is None:
            best_child = node.left
        elif node.left is None:
            best_child = node.right
        else:
            if point[dimension] < node.obj[dimension]:
                best_child = node.left
            else:
                best_child = node.right

        self.nearest_search(best_child, point, max_nodes, best_nodes)

        if len(best_nodes) < max_nodes or own_distance < best_nodes[0][1]:
            self.save_node(node, own_distance, max_nodes, best_nodes)

        if len(best_nodes) < max_nodes or abs(linear_distance) < best_nodes[0][1]:
            if best_child is node.left:
                other_child = node.right
            else:
                other_child = node.left
            if other_child is not None:
                self.nearest_search(other_child, point, max_nodes, best_nodes)

    def save_node(self, node, distance, max_nodes, best_nodes):
        # keep the list sorted, farthest node first
        index = 0
        while index < len(best_nodes) and best_nodes[index][1] > distance:
            index += 1
        best_nodes.insert(index, [node, distance])
        if len(best_nodes) > max_nodes:
            best_nodes.pop(0)


def coordinate_getter(key):
    def get_coordinate(point):
        return point[key]

    return get_coordinate


cam = None

# emoji name -> average color, loaded from data/emoji-average-colors.json
emojis = None
icons = None

# You will have to download the emoji files first
# Please see https://github.com/generative-design/Code-Package-p5.js/tree/master/data
emojis_path = "data/twemoji/"
tree = None

recording = False


def preload():
    global emojis, icons
    with open("data/emoji-average-colors.json") as f:
        emojis = json.load(f)
    icons = {}
    for name in emojis:
        icons[name] = load_image(emojis_path + "16x16/" + name + ".png")


def setup():
    global tree, cam
    size(80 * 16, 60 * 16)
    preload()

    # setup kdTree to find neareast color in a speedy way
    colors = []
    for name in emojis:
        col = emojis[name]["averageColor"]
        col["emoji"] = name
        colors.append(col)
    tree = KdTree(colors, distance, ["r", "g", "b"])

    # setup camera
    cam = create_webcam(80, 60)  # GAP: create_webcam() is missing from the shipped build (the engine's "webcam" feature is off)


def distance(a, b):
    return pow(a["r"] - b["r"], 2) + pow(a["g"] - b["g"], 2) + pow(a["b"] - b["b"], 2)


def draw():
    background(1.0)
    # image(cam, 0, 0, width, width * cam.height / cam.width)
    cam.load_pixels()  # GAP: Webcam.load_pixels() is missing

    title_width = width / cam.width  # GAP: Webcam.width / Webcam.height are missing
    title_height = height / cam.height
    # print("tile size: " + str(title_width) + " x " + str(title_height))
    for grid_x in range(cam.width):
        for grid_y in range(cam.height):
            # grid position
            pos_x = title_width * grid_x
            pos_y = title_height * grid_y
            # get current color
            rgba = cam.get(min(grid_x, cam.width - 1), grid_y)  # GAP: Webcam.get(x, y) is missing
            # find emoji with 'nearest' color
            nearest = tree.nearest({"r": red(rgba) * 255, "g": green(rgba) * 255, "b": blue(rgba) * 255}, 1)[0][0]
            # print(nearest)
            image(icons[nearest["emoji"]], pos_x, pos_y, title_width, title_height)
            # fill(rgba)
            # ellipse(pos_x, pos_y, title_width, title_height)

    if recording:
        save_frame(timestamp() + "_##.png")


def key_released():
    global recording
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == "f" or key == "F":
        recording = not recording


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
