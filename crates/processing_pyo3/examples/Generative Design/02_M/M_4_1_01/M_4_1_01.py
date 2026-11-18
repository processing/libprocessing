# M_4_1_01
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

# some moving nodes
#
# KEYS
# r             : re-initialize velocity vectors of the nodes
# s             : save png
from datetime import datetime

from processing_py import *


class Node:

    def __init__(self, the_x, the_y):
        self.x = the_x
        self.y = the_y

        # velocity
        self.velocity = Vec2()

        # minimum and maximum posiions
        self.min_x = 5
        self.min_y = 5
        self.max_x = width - 5
        self.max_y = height - 5

        # damping of the velocity (0 = no damping, 1 = full damping)
        self.damping = 0.1

    # ------ calculate new position of the node ------
    def update(self):
        self.x += self.velocity.x
        self.y += self.velocity.y

        if self.x < self.min_x:
            self.x = self.min_x - (self.x - self.min_x)
            self.velocity.x = -self.velocity.x
        if self.x > self.max_x:
            self.x = self.max_x - (self.x - self.max_x)
            self.velocity.x = -self.velocity.x

        if self.y < self.min_y:
            self.y = self.min_y - (self.y - self.min_y)
            self.velocity.y = -self.velocity.y
        if self.y > self.max_y:
            self.y = self.max_y - (self.y - self.max_y)
            self.velocity.y = -self.velocity.y

        self.velocity.x *= (1 - self.damping)
        self.velocity.y *= (1 - self.damping)


node_count = 20

# my_nodes array
my_nodes = [None] * node_count


# image output
save_one_frame = False


def setup():
    size(600, 600)
    background(1.0)
    no_stroke()
    fill(0.0)

    # setup my_nodes
    for i in range(node_count):
        my_nodes[i] = Node(random(width), random(height))
        my_nodes[i].velocity.x = random(-3, 3)
        my_nodes[i].velocity.y = random(-3, 3)
        my_nodes[i].damping = 0.01


def draw():
    global save_one_frame
    fill(1.0, 0.05)
    rect(0, 0, width, height)

    for i in range(len(my_nodes)):
        my_nodes[i].update()

        # draw node
        fill(0.0, 1.0)
        ellipse(my_nodes[i].x, my_nodes[i].y, 10, 10)

    # image output
    if save_one_frame == True:
        save_frame("_M_4_1_01_" + timestamp() + ".png")
        save_one_frame = False


def key_pressed():
    global save_one_frame
    if key == "r" or key == "R":
        for i in range(node_count):
            my_nodes[i].velocity.x = random(-5, 5)
            my_nodes[i].velocity.y = random(-5, 5)

    if key == "s" or key == "S":
        save_one_frame = True


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
