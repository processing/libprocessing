# M_4_2_01
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

# a simple attractor
#
# MOUSE
# left click, drag  : attract nodes
#
# KEYS
# r                 : reset nodes
# s                 : save png
from datetime import datetime

from processing_py import *


# Node class of the generativedesign library
class Node:

    def __init__(self, the_x=0, the_y=0, the_z=0):
        self.x = the_x
        self.y = the_y
        self.z = the_z

        # ------   properties ------
        # if needed, an ID for the node
        self.id = ""
        self.diameter = 0

        self.min_x = -float("inf")
        self.max_x = float("inf")
        self.min_y = -float("inf")
        self.max_y = float("inf")
        self.min_z = -float("inf")
        self.max_z = float("inf")

        self.velocity = Vec3()
        self.p_velocity = Vec3()
        self.max_velocity = 10

        self.damping = 0.5
        # radius of impact
        self.radius = 200
        # strength: positive for attraction, negative for repulsion (default for Nodes)
        self.strength = -1
        # parameter that influences the form of the function
        self.ramp = 1.0

    # ------ calculate attraction ------
    def attract_nodes(self, the_nodes):
        # attraction or repulsion part
        for i in range(len(the_nodes)):
            other_node = the_nodes[i]
            # stop when empty
            if other_node == None:
                break
            # not with itself
            if other_node == self:
                continue

            self.attract(other_node)

    def attract(self, the_node):
        d = dist(self.x, self.y, self.z, the_node.x, the_node.y, the_node.z)

        if d > 0 and d < self.radius:
            s = pow(d / self.radius, 1 / self.ramp)
            f = s * 9 * self.strength * (1 / (s + 1) + ((s - 3) / 4)) / d
            df = Vec3(self.x - the_node.x, self.y - the_node.y, self.z - the_node.z)
            df.mult(f)

            the_node.velocity.x += df.x
            the_node.velocity.y += df.y
            the_node.velocity.z += df.z

    # ------ update positions ------
    def update(self, the_lock_x=False, the_lock_y=False, the_lock_z=False):

        self.velocity.limit(self.max_velocity)

        self.p_velocity = self.velocity.copy()

        if not the_lock_x:
            self.x += self.velocity.x
        if not the_lock_y:
            self.y += self.velocity.y
        if not the_lock_z:
            self.z += self.velocity.z

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

        if self.z < self.min_z:
            self.z = self.min_z - (self.z - self.min_z)
            self.velocity.z = -self.velocity.z
        if self.z > self.max_z:
            self.z = self.max_z - (self.z - self.max_z)
            self.velocity.z = -self.velocity.z

        self.velocity.mult(1 - self.damping)

    # ------ getters and setters ------
    def set_id(self, the_id):
        self.id = the_id

    def set_boundary(self, the_min_x, the_min_y, the_max_x, the_max_y):
        self.min_x = the_min_x
        self.max_x = the_max_x
        self.min_y = the_min_y
        self.max_y = the_max_y

    def set_damping(self, the_damping):
        self.damping = the_damping

    def set_radius(self, the_radius):
        self.radius = the_radius

    def set_strength(self, the_strength):
        self.strength = the_strength


class Attractor:

    def __init__(self, the_x, the_y):
        # position
        self.x = the_x
        self.y = the_y

        # radius of impact
        self.radius = 200

    def attract(self, the_node):
        # calculate distance
        dx = self.x - the_node.x
        dy = self.y - the_node.y
        d = mag(dx, dy)

        if d > 0 and d < self.radius:
            # calculate force
            s = d / self.radius
            f = 1 / pow(s, 0.5) - 1
            f = f / self.radius

            # apply force to node velocity
            the_node.velocity.x += dx * f
            the_node.velocity.y += dy * f

x_count = 200
y_count = 200
grid_size = 500

# nodes array
my_nodes = [None] * (x_count * y_count)

# attractor
my_attractor = None


# image output
save_one_frame = False


def setup():
    global my_attractor
    size(600, 600)

    # setup drawing parameters
    no_stroke()

    background(1.0)

    cursor(CROSS)

    # setup node grid
    init_grid()

    # setup attractor
    my_attractor = Attractor(0, 0)


def draw():
    global save_one_frame
    fill(1.0, 0.1)
    rect(0, 0, width, height)

    my_attractor.x = mouse_x
    my_attractor.y = mouse_y

    for i in range(len(my_nodes)):
        if mouse_is_pressed:
            my_attractor.attract(my_nodes[i])

        my_nodes[i].update()

        # draw nodes
        fill(0.0)
        rect(my_nodes[i].x, my_nodes[i].y, 1, 1)

    # image output
    if save_one_frame == True:
        save_frame("_M_4_1_02_" + timestamp() + ".png")
        save_one_frame = False


def init_grid():
    i = 0
    for y in range(y_count):
        for x in range(x_count):
            x_pos = x * (grid_size / (x_count - 1)) + (width - grid_size) / 2
            y_pos = y * (grid_size / (y_count - 1)) + (height - grid_size) / 2
            my_nodes[i] = Node(x_pos, y_pos)
            my_nodes[i].set_boundary(0, 0, width, height)
            my_nodes[i].set_damping(0.02)  # 0.0 - 1.0
            i += 1


def key_pressed():
    global save_one_frame
    if key == "r" or key == "R":
        init_grid()

    if key == "s" or key == "S":
        save_one_frame = True


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
