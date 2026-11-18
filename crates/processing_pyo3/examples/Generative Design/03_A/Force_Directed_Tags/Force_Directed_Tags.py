# Force_Directed_Tags
# ImageRibbon.pde, Tag.pde
#
# Generative Gestaltung, ISBN: 978-3-87439-759-9
# First Edition, Hermann Schmidt, Mainz, 2009
# Hartmut Bohnacker, Benedikt Groß, Julia Laub, Claudius Lazzeroni
# Copyright 2009 Hartmut Bohnacker, Benedikt Groß, Julia Laub, Claudius Lazzeroni
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

# generates a force directed layout with the tags found in book.xml.
# this sketch is not explained in the book and not commented in detail
#
# MOUSE
# drag                : drag nodes
#
# KEYS
# a                   : stop/start attraction
# arrow left/right    : node radius -/+
# arrow down/up       : node strength -/+
# s                   : save png
import xml.etree.ElementTree as ElementTree
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


class Tag:
    def __init__(self, the_name):
        self.name = the_name
        self.count = 0

        # percentage of this tag from all tags (0-1)
        self.ratio = 0
        # position of the tag block (0-1)
        self.position = 0

    def __str__(self):
        return self.name + " (" + str(self.count) + ")"


# initial parameters
qf = 3

draw_springs = True
spring_color = color(0.0, 1.0)
spring_colors = [color(0.0), color(226 / 255, 185 / 255, 0.0), color(0.0, 143 / 255, 193 / 255)]

draw_nodes_1 = True
node_diameter_1 = 0.6
draw_nodes_2 = True
node_diameter_2 = 1

node_radius = 70
node_strength = -5

draw_page_layout = False
page_color = color(0.0, 80 / 255)
draw_blank_images = True
draw_text = True

attraction_on = True

# images
image_count = 0

# pages
images_per_line = 16
grid_margin_left = 9 * qf
grid_margin_top = 10.5 * qf
grid_step_x = 24.314 * qf
grid_step_y = 19.185 * qf
grid_image_w = 18.8 * qf
grid_image_h = 13.1 * qf
grid_fold_w = 14 * qf


# an array for the nodes
nodes = []
# an array for the springs
springs = []

# dragged node
selected_node = None

# tagging
book_xml = None
tag_map = {}
sorted_tags = []
page_tags = []
page_tag_positions = []
page_sum = 0
book_parts = []

font = None


def setup():
    global font, book_xml, page_sum, sorted_tags, image_count, springs
    size((2 * 205) * qf, 285 * qf)
    background(1.0)
    no_stroke()

    font = load_font("data/FreeSansBold.ttf")
    text_font(font)
    text_size(7 * qf)

    # ------ load and interpret xml ------

    book_xml = ElementTree.parse("data/book.xml").getroot()

    tag_sum = 0
    sections = book_xml.findall("sections/section")
    for i in range(len(sections)):
        page_count = int(sections[i].get("pagecount", 0))
        page_name = sections[i].get("name")
        bookpart = int(sections[i].get("part", 0))

        if bookpart > 0:
            book_parts.append(page_sum)

        page_sum += page_count

        tags = sections[i].findall("tag")

        tag_list = []
        in_tag_block_position = []

        for j in range(len(tags)):
            tag_name = tags[j].get("name")
            tag_name_upper_case = tag_name

            tag = tag_map.get(tag_name_upper_case)
            if tag == None:
                tag = Tag(tag_name)
                tag_map[tag_name_upper_case] = tag
            tag.count += page_count
            tag_sum += page_count

            tag_list.append(tag)
            in_tag_block_position.append(tag.count - page_count)

        # append array which holds all tags for each page
        for k in range(page_count):
            page_tags.append(tag_list)

            # print(in_tag_block_position)
            page_tag_positions.append(in_tag_block_position.copy())

            for j in range(len(tags)):
                c = in_tag_block_position[j]
                c += 1
                in_tag_block_position[j] = c

    sorted_tags = list(tag_map.keys())
    sorted_tags.sort()

    image_count = page_sum

    # ------ init nodes and springs ------

    # init nodes
    for i in range(page_sum):
        ix = i % images_per_line
        iy = floor(i / images_per_line)
        px = grid_margin_left + ix * grid_step_x + grid_image_w / 2
        if ix >= images_per_line // 2:
            px += grid_fold_w - (grid_step_x - grid_image_w)
        py = grid_margin_top + iy * grid_step_y + grid_image_h / 2
        n = Node(px, py)
        nodes.append(n)
        nodes[i].set_boundary(px, py, px, py)
        nodes[i].set_radius(50)
        nodes[i].set_strength(-5)
        nodes[i].set_id(str(i))
    for i in range(len(sorted_tags)):
        # print(sorted_tags[i])
        n = Node(width / 2 + random(-200, 200), height / 2 + random(-200, 200))
        nodes.append(n)
        nodes[page_sum + i].set_boundary(0, 0, width, height)
        nodes[page_sum + i].set_radius(node_radius * qf)
        nodes[page_sum + i].set_strength(node_strength)
        nodes[page_sum + i].set_id(sorted_tags[i])

    # set springs
    springs = []

    for j in range(len(page_tags)):
        for i in range(len(page_tags[j])):
            t = page_tags[j][i]
            tag_index = get_tag_index_by_name(t.name)
            # print(t.name + ", " + str(tag_index))
            if tag_index >= 0:
                new_spring = Spring(nodes[j], nodes[len(page_tags) + tag_index])
                new_spring.set_length(20)
                new_spring.set_stiffness(1)
                springs.append(new_spring)


def draw():
    background(1.0)

    if attraction_on:
        # let all nodes repel each other
        for i in range(page_sum, len(nodes)):
            nodes[i].attract_nodes(nodes)
        # apply spring forces
        for i in range(len(springs)):
            springs[i].update()
        # apply velocity vector and update position
        for i in range(page_sum, len(nodes)):
            nodes[i].update()

    if selected_node != None:
        selected_node.x = mouse_x
        selected_node.y = mouse_y

    if draw_page_layout:
        no_fill()
        stroke(0.0, 150 / 255)
        stroke_weight(0.15 * qf)

        for i in range(1, 6):
            line(i * 345 * qf, 0, i * 345 * qf, height)

    if draw_blank_images:
        fill(1.0)
        stroke(page_color)
        stroke_weight(0.15 * qf)

        for i in range(image_count):
            ix = i % images_per_line
            iy = floor(i / images_per_line)
            px = grid_margin_left + ix * grid_step_x
            if ix >= images_per_line // 2:
                px += grid_fold_w - (grid_step_x - grid_image_w)
            py = grid_margin_top + iy * grid_step_y
            rect(px, py, grid_image_w, grid_image_h)
            line(px + grid_image_w / 2, py, px + grid_image_w / 2, py + grid_image_h)

    # draw springs
    if draw_springs:
        stroke_weight(0.1 * qf)
        no_fill()
        act_book_part = 0
        for i in range(len(springs)):
            if act_book_part + 1 < len(book_parts):
                if parse_int(springs[i].from_node.id) >= book_parts[act_book_part + 1]:
                    act_book_part += 1
            stroke(spring_colors[act_book_part])
            x1 = springs[i].from_node.x + springs[i].from_node.z
            y1 = springs[i].from_node.y
            x2 = springs[i].to_node.x + springs[i].to_node.z
            y2 = springs[i].to_node.y
            cxoff = -50 * qf
            if x2 < width / 2:
                cxoff = +50 * qf

            line(x1, y1, x2, y2)

    # draw nodes
    no_stroke()
    act_book_part = 0
    for i in range(len(nodes)):
        if act_book_part + 1 < len(book_parts):
            if parse_int(nodes[i].id) >= book_parts[act_book_part + 1]:
                act_book_part += 1

        node_diameter = node_diameter_1 * qf
        if i >= page_sum:
            tag = tag_map.get(nodes[i].id)
            s = remap(tag.count, 1, 50, 2.5, 15) / 4
            node_diameter = node_diameter_2 * qf * s

        if draw_nodes_1 and i < page_sum:
            fill(spring_colors[act_book_part])
            ellipse(nodes[i].x + nodes[i].z, nodes[i].y, node_diameter_1 * qf, node_diameter_1 * qf)

        if draw_nodes_2 and i >= page_sum:
            fill(1.0)
            ellipse(nodes[i].x + nodes[i].z, nodes[i].y, node_diameter, node_diameter)
            fill(0.0)
            ellipse(nodes[i].x + nodes[i].z, nodes[i].y, node_diameter * 0.7, node_diameter * 0.7)

    if draw_text:
        fill(0.0)
        for i in range(page_sum, len(nodes)):
            tag = tag_map.get(nodes[i].id)
            s = remap(tag.count, 1, 50, 2.5, 15)

            text_font(font)
            text_size(s * qf)
            if nodes[i].x + nodes[i].z > width / 2:
                text_align(LEFT, CENTER)
                text(nodes[i].id, nodes[i].x + nodes[i].z + (s * 0.1 + 1) * qf, nodes[i].y)
            else:
                text_align(RIGHT, CENTER)
                text(nodes[i].id, nodes[i].x + nodes[i].z - (s * 0.1 + 1) * qf, nodes[i].y)


def get_tag_index_by_name(name):
    for i in range(len(sorted_tags)):
        s1 = name.upper()
        s2 = sorted_tags[i].upper()
        if s1 == s2:
            return i
    return -1


# Processing's int(String): 0 when the string is not a number
def parse_int(s):
    if s.isdigit():
        return int(s)
    return 0


def mouse_pressed():
    global selected_node
    # Ignore anything greater than this distance
    max_dist = 20
    for i in range(page_sum, len(nodes)):
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
    global attraction_on, node_radius, node_strength
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key == "a" or key == "A":
        attraction_on = not attraction_on

    if key_code == LEFT_ARROW:
        node_radius /= 1.5
    if key_code == RIGHT_ARROW:
        node_radius *= 1.5
    if key_code == DOWN:
        node_strength /= 1.5
    if key_code == UP:
        node_strength *= 1.5
    print("nodeRadius: " + str(node_radius) + ", " + "nodeStrength: " + str(node_strength))

    for i in range(len(sorted_tags)):
        nodes[page_sum + i].set_radius(node_radius * qf)
        nodes[page_sum + i].set_strength(node_strength)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
