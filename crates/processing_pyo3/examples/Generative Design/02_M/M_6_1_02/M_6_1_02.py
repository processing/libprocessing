# M_6_1_02
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

# two nodes and a spring
#
# MOUSE
# click, drag   : postion of one of the nodes
#
# KEYS
# s             : save png
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


# Spring class of the generativedesign library
class Spring:

    # ------ constructors ------
    def __init__(self, the_from_node, the_to_node, the_length=100, the_stiffness=0.6, the_damping=0.9):
        self.from_node = the_from_node
        self.to_node = the_to_node

        self.length = the_length
        self.stiffness = the_stiffness
        self.damping = the_damping

    # ------ apply forces on spring and attached nodes ------
    def update(self):
        # calculate the target position
        # target = normalize(to - from) * length + from
        diff = Vec3(self.to_node.x - self.from_node.x, self.to_node.y - self.from_node.y, self.to_node.z - self.from_node.z)
        diff = diff.normalize()
        diff.mult(self.length)
        target = Vec3(self.from_node.x + diff.x, self.from_node.y + diff.y, self.from_node.z + diff.z)

        force = Vec3(target.x - self.to_node.x, target.y - self.to_node.y, target.z - self.to_node.z)
        force.mult(0.5)
        force.mult(self.stiffness)
        force.mult(1 - self.damping)

        self.to_node.velocity.add(force)
        self.from_node.velocity.add(force * -1)

    # ------ getters and setters ------
    def set_length(self, the_length):
        self.length = the_length

    def set_stiffness(self, the_stiffness):
        self.stiffness = the_stiffness

    def set_damping(self, the_damping):
        self.damping = the_damping


node_a = None
node_b = None
spring = None


def setup():
    global node_a, node_b, spring
    size(600, 600)
    fill(0.0)

    node_a = Node(width / 2 + random(-50, 50), height / 2 + random(-50, 50))
    node_b = Node(width / 2 + random(-50, 50), height / 2 + random(-50, 50))

    node_a.set_damping(0.1)
    node_b.set_damping(0.1)

    spring = Spring(node_a, node_b)
    spring.set_length(100)
    spring.set_stiffness(0.6)
    spring.set_damping(0.3)


def draw():
    background(1.0)

    if mouse_is_pressed == True:
        node_a.x = mouse_x
        node_a.y = mouse_y

    # update spring
    spring.update()

    # update node positions
    node_a.update()
    node_b.update()

    # draw spring
    stroke(0.0, 130 / 255, 164 / 255)
    stroke_weight(4)
    line(node_a.x, node_a.y, node_b.x, node_b.y)

    # draw nodes
    no_stroke()
    fill(0.0)
    ellipse(node_a.x, node_a.y, 20, 20)
    ellipse(node_b.x, node_b.y, 20, 20)


def key_pressed():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
