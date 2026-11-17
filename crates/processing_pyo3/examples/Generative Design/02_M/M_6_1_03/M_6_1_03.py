# M_6_1_03
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

# more nodes and more springs
#
# KEYS
# r             : reset positions
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


# an array for the nodes
nodes = [None] * 100
# an array for the springs
springs = []

# dragged node
selected_node = None

node_diameter = 16


def setup():
    size(800, 800)
    background(1.0)
    no_stroke()

    init_nodes_and_springs()


def draw():
    background(1.0)
    #  fill(1.0, 20 / 255)
    #  rect(0, 0, width, height)

    # let all nodes repel each other
    for i in range(len(nodes)):
        nodes[i].attract_nodes(nodes)
    # apply spring forces
    for i in range(len(springs)):
        springs[i].update()
    # apply velocity vector and update position
    for i in range(len(nodes)):
        nodes[i].update()

    if selected_node != None:
        selected_node.x = mouse_x
        selected_node.y = mouse_y

    # draw nodes
    stroke(0.0, 130 / 255, 164 / 255)
    stroke_weight(2)
    for i in range(len(springs)):
        line(springs[i].from_node.x, springs[i].from_node.y, springs[i].to_node.x, springs[i].to_node.y)
    # draw nodes
    no_stroke()
    for i in range(len(nodes)):
        fill(1.0)
        ellipse(nodes[i].x, nodes[i].y, node_diameter, node_diameter)
        fill(0.0)
        ellipse(nodes[i].x, nodes[i].y, node_diameter - 4, node_diameter - 4)


def init_nodes_and_springs():
    global springs
    # init nodes
    rad = node_diameter / 2
    for i in range(len(nodes)):
        nodes[i] = Node(width / 2 + random(-200, 200), height / 2 + random(-200, 200))
        nodes[i].set_boundary(rad, rad, width - rad, height - rad)
        nodes[i].set_radius(100)
        nodes[i].set_strength(-5)

    # set springs randomly
    springs = []

    for j in range(len(nodes) - 1):
        r_count = floor(random(1, 2))
        for i in range(r_count):
            r = floor(random(j + 1, len(nodes)))
            new_spring = Spring(nodes[j], nodes[r])
            new_spring.set_length(20)
            new_spring.set_stiffness(1)
            springs.append(new_spring)


def mouse_pressed():
    global selected_node
    # Ignore anything greater than this distance
    max_dist = 20
    for i in range(len(nodes)):
        check_node = nodes[i]
        d = dist(mouse_x, mouse_y, check_node.x, check_node.y)
        if d < max_dist:
            selected_node = check_node
            max_dist = d


def mouse_released():
    global selected_node
    if selected_node != None:
        selected_node = None


def key_pressed():
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "r" or key == "R":
        background(1.0)
        init_nodes_and_springs()


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
