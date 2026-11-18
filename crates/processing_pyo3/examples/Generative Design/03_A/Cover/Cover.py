# Cover
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

# sketch, generating the cover of the book "Generative Gestaltung"
# and automatically saving it as a png.
# attention: this will take some time to render
import os
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


# an ImageRibbon represents an elasic line made of nodes and springs.
# we've built in nodes and springs to have the possibility to implement attraction
# or repulsion, but this was not used in the end.
# the colors of the line are defined by the pixels of an image.
class ImageRibbon:
    def __init__(self, init_points, the_image, the_xml, the_page_in_section, the_strip_index):
        # drawing of colors in strip:  1: normal, -1: reversed, 0: back and forth
        self.direction = 1

        # number of points that are used to draw a spring
        self.detail = 20

        self.count = len(init_points)
        self.source_image = the_image
        self.xml = the_xml
        self.page_in_section = the_page_in_section
        self.strip_index = the_strip_index

        c1 = 0
        c2 = 0
        c3 = 0
        self.colors = [None] * (self.count * self.detail)
        w = self.source_image.width
        h = self.source_image.height
        step = (w * h) / float(self.count * self.detail)
        for i in range(self.count * self.detail):
            self.colors[i] = self.source_image.pixels[int(i * step)]

        self.colors = sort_colors_grayscale(self.colors)

        self.nodes = [None] * self.count
        self.springs = [None] * (self.count - 1)

        for i in range(self.count):
            self.nodes[i] = Node(init_points[i].x, init_points[i].y, 0)
            self.nodes[i].strength = -0.05
        for i in range(self.count - 1):
            self.springs[i] = Spring(self.nodes[i], self.nodes[i + 1])
            self.springs[i].length = 2

    def attract(self, the_nodes):
        for i in range(self.count):
            self.nodes[i].attract_nodes(the_nodes)

    def update(self):
        for i in range(self.count - 1):
            # self.springs[i].update()
            pass
        for i in range(1, self.count - 1):
            self.nodes[i].update(True, False, True)

    def draw(self):
        push_style()

        no_stroke()

        nodes = self.nodes
        count = self.count
        detail = self.detail
        colors = self.colors

        old_x = nodes[0].x
        old_y = nodes[0].y
        old_x1 = nodes[0].x
        old_y1 = nodes[0].y - 0.5
        old_x2 = nodes[0].x
        old_y2 = nodes[0].y + 0.5

        c1j = 0
        a1j = 0
        a2j = 0
        c2j = 0
        pos_x = 0
        pos_y = 0

        for j in range(count - 1):
            for i in range(detail):
                t_all = remap(j * detail + i, 0, count * detail, 0, 1)
                t = remap(i, 0, detail, 0, 1)

                c1j = max(j - 1, 0)
                a1j = j
                a2j = min(j + 1, count - 1)
                c2j = min(j + 2, count - 1)
                pos_x = curve_point(nodes[c1j].x, nodes[a1j].x, nodes[a2j].x, nodes[c2j].x, t)
                pos_y = curve_point(nodes[c1j].y, nodes[a1j].y, nodes[a2j].y, nodes[c2j].y, t)

                c = color(0.0)
                a = 1

                ci = count * detail - (j * detail + i) - 1
                if self.direction == -1:
                    ci = len(colors) - ci - 1
                if self.direction == 0:
                    if ci < len(colors) / 2.0:
                        ci = int(remap(ci, 0, len(colors) / 2.0, len(colors) - 1, 0))
                    else:
                        ci = int(remap(ci, len(colors) / 2.0, len(colors) - 1, 0, len(colors) - 1))

                c = colors[ci]
                a = sqrt(sq(saturation(c)) + sq(1 - brightness(c)))
                a = min(a, 1)
                fill(c.with_alpha(a))

                dx = pos_x - old_x
                dy = pos_y - old_y
                n = Vec2(-dy, dx)
                if n.mag() > 0:
                    n = n.normalize()
                f = remap(t_all, 0, 1, 1.5, 0.5) * qf / 2
                n.x *= f
                n.y *= f

                if j == 0 and i <= 1:
                    old_x = nodes[0].x
                    old_y = nodes[0].y
                    old_x1 = nodes[0].x - n.x
                    old_y1 = nodes[0].y - n.y
                    old_x2 = nodes[0].x + n.x
                    old_y2 = nodes[0].y + n.y

                begin_shape()
                vertex(old_x1, old_y1)
                vertex(pos_x - n.x, pos_y - n.y)
                vertex(pos_x + n.x, pos_y + n.y)
                vertex(old_x2, old_y2)
                end_shape()

                old_x = pos_x
                old_y = pos_y
                old_x1 = pos_x - n.x
                old_y1 = pos_y - n.y
                old_x2 = pos_x + n.x
                old_y2 = pos_y + n.y
        pop_style()

    def draw_end_point(self):
        no_stroke()

        j = self.count - 2
        i = self.detail - 1

        pos_x = self.nodes[j + 1].x
        pos_y = self.nodes[j + 1].y

        c = color(0.0)
        a = 1

        ci = self.count * self.detail - (j * self.detail + i) - 1
        if self.direction == -1:
            ci = len(self.colors) - ci - 1
        if self.direction == 0:
            if ci < len(self.colors) / 2.0:
                ci = int(remap(ci, 0, len(self.colors) / 2.0, len(self.colors) - 1, 0))
            else:
                ci = int(remap(ci, len(self.colors) / 2.0, len(self.colors) - 1, 0, len(self.colors) - 1))

        c = self.colors[ci]
        a = red(c) * 0.3 + green(c) * 0.59 + blue(c) * 0.11
        fill(c.with_alpha(1 - a))

        ellipse(pos_x, pos_y, 4, 4)


# GenerativeDesign.sortColors(colors, GenerativeDesign.GRAYSCALE) of the generativedesign library
def sort_colors_grayscale(colors):
    return sorted(colors, key=grayscale_of)


def grayscale_of(c):
    return red(c) * 0.222 + green(c) * 0.707 + blue(c) * 0.071


# an instance of Tag collects information about one of the tags that appear in
# "book.xml".
class Tag:
    def __init__(self, the_name):
        self.name = the_name
        # how often does this tag appear
        self.count = 0

        # which orderNum inside all sorted tags
        self.order_num = 0
        # percentage of this tag from all tags (0-1)
        self.ratio = 0
        # position of the tag block (0-1)
        self.position = 0

    def __str__(self):
        return self.name + "(" + str(self.count) + ")"


# initial parameters
qf = 1  # quality factor: 5 for print resolution
qffinal = 5
strip_resolution = 40
strip_count = 20

block_factor = 0.56
block_offset_x = 580 * qf
block_offset_y = 306 * qf
tag_block_width = 207 * qf
on_black = False
strip_dir = 0
strip_dir_alternating = False

reversed = False
sort_ribbons_on_start_x_pos = 620 * qf
sort_ribbons_on_start_y_pos = 630 * qf

# typo points
typed_texts = ["GENERATIVE", "GESTALTUNG", "Entwerfen\nProgrammieren", "Visualisieren", "Hartmut Bohnacker\nBenedikt Groß\nJulia Laub\nClaudius Lazzeroni (Hrsg.)", "Verlag Hermann\nSchmidt Mainz", "............................."]
font_names = ["FreeSansBold.ttf", "FreeSansBold.ttf", "FreeSansBold.ttf", "FreeSansBold.ttf", "FreeSansBold.ttf", "FreeSansBold.ttf", "FreeSansBold.ttf"]
font_sizes = [72, 72, 25, 25, 14, 20, 14, 12]
line_spacings = [0.9, 0.9, 1.1, 1.1, 1.2, 1.2, 1]
typo_segment_lengths = [3.5, 3.5, 3.5, 3.5, 3.5, 3.5, 3.5]
text_pos_xs = [205, 205, 448.5, 449.5, 261, 505, 505]
text_pos_ys = [188, 256, 313, 370, 424, 730, 752]
text_scale_xs = [1, 1, 1, 1, 1, 1, 1]
text_angles = [0, 0, 0, 0, 0, 0, 0]
text_normals_factor = [1.5, 1.5, 0.75, 0.75, 0.75, 0.75, 0.75]

strip_distribution_potenz = 1.27

persp_center_x = 630 * qf
persp_center_y = 630 * qf


letter_y = 0
font = None
pnts = None
typo_points = []
typo_next_index = []
typo_prev_index = []
typo_normals = []
typo_tangents = []
typo_nodes = []
text_index_starts = []


# images
img = None
images = None
image_count = 0


# ribbons
ribbons = None

# attractors
my_attractors = [None] * 2


# tagging
tag_font = None
book_xml = None
tag_map = {}
sorted_tags = None
page_xmls = []
page_in_section = []
page_tags = []
page_tag_positions = []
tag_block_factor = 1
tag_block_gap_ratio = 0.25


def setup():
    global tag_font, font, pnts, letter_y, typo_prev_index, book_xml, sorted_tags
    global image_count, images, img, ribbons, strip_dir
    size(1310 * qf, 822 * qf)
    frame_rate(4)

    tag_font = load_font("data/FreeSansBold.ttf")

    # ------ init text points ------

    for ti in range(len(typed_texts)):
        typed_text = typed_texts[ti]
        font_name = font_names[ti]
        font_size = font_sizes[ti] * qffinal
        line_spacing = font_sizes[ti] * qf * line_spacings[ti]
        typo_segment_length = typo_segment_lengths[ti] * qffinal
        text_pos_x = text_pos_xs[ti] * qf
        text_pos_y = text_pos_ys[ti] * qf
        text_scale_x = text_scale_xs[ti]
        text_angle = radians(text_angles[ti])

        font = load_font("data/" + font_name)
        text_font(font)
        text_size(font_size)
        text_align(LEFT)

        text_index_starts.append(len(typo_nodes))

        first_index = 0
        texts = typed_text.split("\n")
        for i in range(len(texts)):
            # points on the outlines, one every typo_segment_length
            pnts = text_to_points(texts[i], 0, 0, 1 / typo_segment_length)

            old_pnt = None
            act_pnt = None

            for p in range(len(pnts)):
                pnts[p] = Vec2(pnts[p][0], pnts[p][1])
                pnts[p].x = pnts[p].x / qffinal * qf
                pnts[p].y = pnts[p].y / qffinal * qf

                if act_pnt == None:
                    act_pnt = Vec2(pnts[p].x, pnts[p].y)
                if old_pnt == None:
                    old_pnt = Vec2(pnts[p].x, pnts[p].y)

                # find out, if a new closed shape begins
                act_pnt = Vec2(pnts[p].x, pnts[p].y)

                if p == len(pnts) - 1:
                    typo_next_index.append(first_index)
                    first_index = len(typo_next_index)
                elif dist(act_pnt.x, act_pnt.y, old_pnt.x, old_pnt.y) > typo_segment_length + 1:
                    typo_next_index[len(typo_next_index) - 1] = first_index
                    first_index = len(typo_next_index)
                    typo_next_index.append(len(typo_next_index) + 1)
                else:
                    typo_next_index.append(len(typo_next_index) + 1)
                old_pnt = Vec2(pnts[p].x, pnts[p].y)

                # move typo point to final position and append array
                px = pnts[p].x * text_scale_x
                py = pnts[p].y + letter_y
                pnts[p].x = px * cos(text_angle) - py * sin(text_angle) + text_pos_x
                pnts[p].y = px * sin(text_angle) + py * cos(text_angle) + text_pos_y
                typo_points.append(pnts[p])

                n = Node(pnts[p].x, pnts[p].y)
                # n.id = "fixed"
                n.radius = 100
                n.strength = -1
                n.ramp = 1
                typo_nodes.append(n)
            letter_y += line_spacing
        letter_y = 0

    typo_prev_index = [0] * len(typo_next_index)
    for i in range(len(typo_next_index)):
        typo_prev_index[typo_next_index[i]] = i

    # calculate normals for typo points
    for i in range(len(typo_points)):
        tangent = Vec2(0, 0)
        tangent.x = typo_points[typo_prev_index[i]].x - typo_points[typo_next_index[i]].x
        tangent.y = typo_points[typo_prev_index[i]].y - typo_points[typo_next_index[i]].y
        if tangent.mag() > 0:
            tangent = tangent.normalize()
        typo_tangents.append(tangent)
        typo_normals.append(Vec2(-tangent.y, tangent.x))

    # print(text_index_starts)
    # print(len(typo_nodes))

    # ------ load and interpret xml ------

    book_xml = ElementTree.parse("data/book.xml").getroot()

    page_sum = 0
    tag_sum = 0
    sections = book_xml.findall("sections/section")

    for i in range(len(sections)):
        page_count = int(sections[i].get("pagecount", 0))
        page_name = sections[i].get("name")
        page_sum += page_count

        tags = sections[i].findall("tag")

        tag_list = []
        in_tag_block_position = []

        for j in range(len(tags)):
            tag_name = tags[j].get("name")
            tag_name_upper_case = tag_name.upper()

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
            page_in_section.append(k)
            page_xmls.append(sections[i])

            page_tags.append(tag_list)

            # print(in_tag_block_position)
            page_tag_positions.append(in_tag_block_position.copy())

            for j in range(len(tags)):
                c = in_tag_block_position[j]
                c += 1
                in_tag_block_position[j] = c

    sorted_tags = list(tag_map.keys())
    sorted_tags.sort()

    # distribute tag block ratio and positions
    act_pos = 0
    for i in range(len(sorted_tags)):
        tag_name = sorted_tags[i]
        tag = tag_map.get(tag_name)
        tag.ratio = float(tag.count) / tag_sum
        tag.position = act_pos
        tag.order_num = i
        act_pos += tag.ratio

    # ------ load files ------
    image_count = 0

    dir = "data/images"

    if os.path.isdir(dir):
        contents = sorted(os.listdir(dir))
        images = [None] * len(contents)
        for i in range(len(contents)):
            if contents[i][0] == ".":
                continue
            elif contents[i].lower().endswith(".png") or contents[i].lower().endswith(".jpg"):
                child_file = os.path.join(dir, contents[i])
                images[image_count] = load_image(child_file)
                print(str(image_count) + " - " + contents[i])
                image_count += 1

    # ------ setup ribbons ------
    ribbons = [None] * (image_count * strip_count)
    ii = 0
    act_text_index = 0
    noise_seed(2)

    for i in range(image_count):
        for s in range(strip_count):
            img = images[i]
            w = img.width
            h = img.height // strip_count
            y = int(s * img.height / float(strip_count))
            strip = img.get(0, y, w, h)  # GAP: Image.get(x, y, w, h) returning a sub-image is missing

            pi = remap(i * strip_count + s, 0, image_count * strip_count, 0, 1)
            pi = pow(pi, strip_distribution_potenz)
            p_index = int(remap(pi, 0, 1, 1, len(typo_points) - 1))
            p_index = p_index % (len(typo_points) - 1) + 1
            if reversed:
                p_index = len(typo_points) - 1 - p_index

            if act_text_index < len(text_index_starts) - 1:
                if p_index >= text_index_starts[act_text_index + 1]:
                    act_text_index += 1

            # get position from tag information
            tag_index = s % len(page_tags[i])
            strip_tag = page_tags[i][tag_index]
            strip_tag_position = page_tag_positions[i][tag_index]
            strip_tag_position /= float(strip_tag.count)
            tag_pos_y = (strip_tag.position + strip_tag_position * strip_tag.ratio * tag_block_factor) * (1 - tag_block_gap_ratio) + strip_tag.order_num * (1.0 / len(sorted_tags)) * tag_block_gap_ratio
            tag_x = -tag_block_width
            tag_y = tag_pos_y * height * block_factor + block_offset_y

            mid_x = 0
            mid_y = (i + float(s) / strip_count) * float(height) / image_count * block_factor + block_offset_y

            typo_x = typo_points[p_index].x - typo_normals[p_index].x * text_normals_factor[act_text_index] * qf
            typo_y = typo_points[p_index].y - typo_normals[p_index].y * text_normals_factor[act_text_index] * qf

            pnts = [None] * strip_resolution
            for p in range(len(pnts)):
                t = remap(p, 0, len(pnts) - 1, 0, 1)
                pos_x = 0
                pos_y = 0

                if t >= 0 and t <= 0.5:
                    tt = remap(t, 0, 0.5, 0, 1)
                    pos_x = bezier_point(tag_x, tag_x + tag_block_width * 0.75, mid_x - tag_block_width * 0.75, mid_x, tt)
                    pos_y = bezier_point(tag_y, tag_y, mid_y, mid_y, tt)
                else:
                    tt = remap(t, 0.5, 1, 0, 1)
                    offx = -(typo_x - persp_center_x)
                    offy = -(typo_y - persp_center_y)
                    pos_x = bezier_point(mid_x, mid_x + 200 * qf, typo_x + offx, typo_x, tt)
                    pos_y = bezier_point(mid_y, mid_y, typo_y + offy, typo_y, tt)

                pnts[p] = Vec2(pos_x, pos_y)

            pnts.reverse()
            ribbons[i * strip_count + s] = ImageRibbon(pnts, strip, page_xmls[i], page_in_section[i], s)
            ribbons[i * strip_count + s].direction = strip_dir
            if strip_dir_alternating:
                strip_dir *= -1
            ii += 1

    sort_ribbons_on_start_dist_to(sort_ribbons_on_start_x_pos, sort_ribbons_on_start_y_pos)


def draw():
    global letter_y
    background(1.0)

    translate(block_offset_x, 0)

    random_seed(0)

    # write tags
    push_style()
    text_align(RIGHT, TOP)
    for i in range(len(sorted_tags)):
        tag_name = sorted_tags[i]
        tag = tag_map.get(tag_name)

        px = -tag_block_width - 8 * qf
        py = tag.position * (1 - tag_block_gap_ratio) + tag.order_num * (1.0 / len(sorted_tags)) * tag_block_gap_ratio
        py = py * height * block_factor + block_offset_y

        no_stroke()
        fill(0.0)
        block_height = tag.ratio * height * block_factor * tag_block_factor * (1 - tag_block_gap_ratio)
        text_font(tag_font)
        text_size(max(block_height * 1, 3.5 * qf))
        text(tag.name, px - 0 * qf, py - block_height * 0.2)
    pop_style()

    # draw ribbons
    for i in range(len(ribbons)):
        print(str(i) + " / " + str(len(ribbons)))
        ribbons[i].draw()

    # draw white typo
    no_stroke()
    fill(1.0)

    for ti in range(len(typed_texts)):
        typed_text = typed_texts[ti]
        font_name = font_names[ti]
        font_size = font_sizes[ti] * qf
        line_spacing = font_size * line_spacings[ti]
        text_pos_x = text_pos_xs[ti] * qf
        text_pos_y = text_pos_ys[ti] * qf
        text_angle = radians(text_angles[ti])

        f = load_font("data/" + font_name)
        text_font(f)
        text_size(font_size - 1)

        texts = typed_text.split("\n")
        letter_y = 0
        push_matrix()
        translate(text_pos_x, text_pos_y)
        rotate(text_angle)
        for i in range(len(texts)):
            text(texts[i], 0, letter_y)
            letter_y += line_spacing
        pop_matrix()
        letter_y = 0

    # automatically save png
    save_frame(timestamp() + ".png")

    print("finished")
    no_loop()

    # save_frame(timestamp() + "_##.png")


def sort_ribbons_on_start_x(dir):
    if dir != 0:
        for j in range(len(ribbons) - 1):
            for i in range(j, len(ribbons)):
                if dir * ribbons[j].nodes[0].x < dir * ribbons[i].nodes[0].x:
                    rj = ribbons[j]
                    ri = ribbons[i]
                    ribbons[j] = ri
                    ribbons[i] = rj


def sort_ribbons_on_start_y(dir):
    if dir != 0:
        for j in range(len(ribbons) - 1):
            for i in range(j, len(ribbons)):
                if dir * ribbons[j].nodes[0].y < dir * ribbons[i].nodes[0].y:
                    rj = ribbons[j]
                    ri = ribbons[i]
                    ribbons[j] = ri
                    ribbons[i] = rj


def sort_ribbons_on_start_y_pos_to(pos):
    for j in range(len(ribbons) - 1):
        for i in range(j, len(ribbons)):
            if abs(ribbons[j].nodes[0].y - pos) < abs(ribbons[i].nodes[0].y - pos):
                rj = ribbons[j]
                ri = ribbons[i]
                ribbons[j] = ri
                ribbons[i] = rj


def sort_ribbons_on_start_dist_to(pos_x, pos_y):
    for j in range(len(ribbons) - 1):
        for i in range(j, len(ribbons)):
            if dist(ribbons[j].nodes[0].x, ribbons[j].nodes[0].y * 5, pos_x, pos_y * 5) < dist(ribbons[i].nodes[0].x, ribbons[i].nodes[0].y * 5, pos_x, pos_y * 5):
                rj = ribbons[j]
                ri = ribbons[i]
                ribbons[j] = ri
                ribbons[i] = rj


def sort_ribbons_on_end_x(dir):
    if dir != 0:
        for j in range(len(ribbons) - 1):
            for i in range(j, len(ribbons)):
                if dir * ribbons[j].nodes[ribbons[j].count - 1].x < dir * ribbons[i].nodes[ribbons[i].count - 1].x:
                    rj = ribbons[j]
                    ri = ribbons[i]
                    ribbons[j] = ri
                    ribbons[i] = rj


def sort_ribbons_on_end_y(dir):
    if dir != 0:
        for j in range(len(ribbons) - 1):
            for i in range(j, len(ribbons)):
                if dir * ribbons[j].nodes[ribbons[j].count - 1].y < dir * ribbons[i].nodes[ribbons[i].count - 1].y:
                    rj = ribbons[j]
                    ri = ribbons[i]
                    ribbons[j] = ri
                    ribbons[i] = rj


def sort_ribbons_on_end_y_pos(pos):
    for j in range(len(ribbons) - 1):
        for i in range(j, len(ribbons)):
            if abs(ribbons[j].nodes[ribbons[j].count - 1].y - pos) < abs(ribbons[i].nodes[ribbons[i].count - 1].y - pos):
                rj = ribbons[j]
                ri = ribbons[i]
                ribbons[j] = ri
                ribbons[i] = rj


def hill_curve(t, parts):
    res = 1
    if t < 1.0 / parts:
        res = -cos(t * PI * parts) * 0.5 + 0.5
    if t > 1 - 1.0 / parts:
        res = -cos((t - 1) * PI * parts) * 0.5 + 0.5
    return res


def int_array_to_string(arr):
    s = ""
    for i in range(len(arr)):
        s = s + str(arr[i]) + ", "
    return s


def float_array_to_string(arr):
    s = ""
    for i in range(len(arr)):
        s = s + str(arr[i]) + ", "
    return s


def array_to_string(arr):
    s = ""
    for i in range(len(arr)):
        s = s + str(arr[i]) + ", "
    return s


# timestamp
def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
