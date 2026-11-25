# M_6_4_01_TOOL
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

# exploring wikipedia
#
# MOUSE
# left click          : show links for the clicked node
# double left click   : create node
# shift + left click  : remove node
# alt + left click    : show backlinks for the clicked node
# right click + drag  : drag canvas or node
# double right click  : open article in browser
#
# KEYS
# 1                   : toggle colorize nodes
# 2                   : toggle fish eye view
# arrow up/down       : zoom
# s                   : save png
#
# The controlP5 panel of the original is not ported; its sliders and toggles
# are the variables below. The text field for the name of a new node is
# replaced by typing the name and pressing enter.
#
# The original loads XML with GenerativeDesign.loadXMLAsync(); here the same
# queries are sent to the current MediaWiki API with urllib in background
# threads and the answers are read as JSON.
import json
import re
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime
from html.parser import HTMLParser
from random import shuffle

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


# Wikipedia only allows a few requests per minute, so there is only one
# request at a time
request_lock = threading.Lock()


# returns the answer of the server, or None if it could not be loaded
def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": "M_6_4_01_TOOL (Generative Design sketch)"})
    request_lock.acquire()
    try:
        for attempt in range(10):
            try:
                response = urllib.request.urlopen(request, timeout=10)
                return response.read()
            except urllib.error.HTTPError as error:
                print(error)
                if error.code != 429:
                    break
                # too many requests: wait as long as Wikipedia asks for and try again
                time.sleep(float(error.headers.get("Retry-After", 5)))
            except Exception as exception:
                print(exception)
                break
    finally:
        request_lock.release()
    return None


# loadXMLAsync of the generativedesign library: returns an empty dictionary
# at once and fills it in the background as soon as the answer has arrived
def load_json_async(url):
    result = {}
    thread = threading.Thread(target=load_json, args=(url, result))
    thread.daemon = True
    thread.start()
    return result


def load_json(url, result):
    data = fetch(url)
    if data != None:
        result.update(json.loads(data))


# collects the text of a html page (everything that is not a tag)
class ContentParser(HTMLParser):

    def __init__(self, the_content):
        HTMLParser.__init__(self)
        self.content = the_content

    def handle_data(self, data):
        self.content.append(data)


# loadHTMLAsync of the generativedesign library: returns an empty list at once
# and fills it in the background with the text content of the page
def load_html_async(url):
    result = []
    thread = threading.Thread(target=load_html, args=(url, result))
    thread.daemon = True
    thread.start()
    return result


def load_html(url, result):
    data = fetch(url)
    if data != None:
        content = []
        parser = ContentParser(content)
        parser.feed(data.decode("utf-8"))
        result.extend(content)


class WikipediaNode(Node):

    def __init__(self, the_graph, the_x=0, the_y=0, the_z=0):
        Node.__init__(self, the_x, the_y, the_z)

        # screen position and screen scaling factor
        self.sx = 0
        self.sy = 0
        self.sfactor = 1

        # reference to the force directed graph
        self.graph = the_graph
        # last activation (rollover) time
        self.activation_time = 0
        # is this a node that was clicked on
        self.was_clicked = False
        # this will contain the respective html as a list (each piece of text as a string)
        self.html_list = []
        self.html_string = ""
        self.html_loaded = False

        # available links
        self.available_links_loaded = False
        self.available_links = []
        self.links_json = None
        # number of links already shown
        self.link_count = 0
        self.ring_radius = 0

        # available backlinks
        self.available_backlinks_loaded = False
        self.available_backlinks = []
        self.backlinks_json = None
        # number of links pointing to this node
        self.backlink_count = 0

        # size of the displayed text
        self.textsize = 18
        # color of the node
        self.node_color = color(0.0)

        self.init()

    def init(self):
        self.activation_time = millis()

    def set_id(self, the_id):
        Node.set_id(self, the_id)

        if the_id != "":
            # load html
            self.html_loaded = False
            self.html_string = ""
            url = encode_url("https://en.wikipedia.org/wiki/" + self.id)
            self.html_list = load_html_async(url)

            # load available links
            self.available_links_loaded = False
            self.available_links = []
            url = encode_url("https://en.wikipedia.org/w/api.php?format=json&formatversion=2&action=query&prop=links&titles=" + self.id + "&pllimit=500&plnamespace=0&redirects")
            self.links_json = load_json_async(url)

            # load available backlinks
            self.available_backlinks_loaded = False
            self.available_backlinks = []
            url = encode_url("https://en.wikipedia.org/w/api.php?format=json&formatversion=2&action=query&list=backlinks&bltitle=" + self.id + "&bllimit=500&blnamespace=0&blredirect")
            self.backlinks_json = load_json_async(url)

    def loader_loop(self):
        # handle loading of html
        if self.id != "":
            if not self.html_loaded:
                if len(self.html_list) > 0 and len(self.html_string) == 0:
                    self.html_string = ", ".join(self.html_list)
                    self.html_loaded = True
                    self.node_color = self.graph.text_to_color(self.html_string)

            # handle loading of links
            if not self.available_links_loaded:
                if len(self.links_json) > 0:
                    # get titles of links
                    children = []
                    if "query" in self.links_json:
                        page = self.links_json["query"]["pages"][0]
                        if "links" in page:
                            children = page["links"]
                    for i in range(1, len(children)):
                        title = children[i]["title"]
                        self.available_links.append(title)

                    querycontinue = self.links_json.get("continue")
                    if querycontinue == None:
                        self.available_links_loaded = True
                        shuffle(self.available_links)
                    else:
                        plcontinue = querycontinue["plcontinue"]
                        url = encode_url("https://en.wikipedia.org/w/api.php?format=json&formatversion=2&action=query&prop=links&titles=" + self.id + "&pllimit=500&plnamespace=0&plcontinue=" + plcontinue)
                        self.links_json = load_json_async(url)

            # handle loading of backlinks
            if not self.available_backlinks_loaded:
                if len(self.backlinks_json) > 0:
                    children = []
                    if "query" in self.backlinks_json:
                        children = self.backlinks_json["query"]["backlinks"]
                    for i in range(1, len(children)):
                        title = children[i]["title"]
                        self.available_backlinks.append(title)

                    querycontinue = self.backlinks_json.get("continue")
                    if querycontinue == None:
                        self.available_backlinks_loaded = True
                        shuffle(self.available_backlinks)
                    else:
                        blcontinue = querycontinue["blcontinue"]
                        url = encode_url("https://en.wikipedia.org/w/api.php?format=json&formatversion=2&action=query&list=backlinks&bltitle=" + self.id + "&bllimit=500&blnamespace=0&blcontinue=" + blcontinue)
                        self.backlinks_json = load_json_async(url)

    def update(self):
        Node.update(self)

        spos = self.graph.screen_pos(self)
        self.sx = spos.x
        self.sy = spos.y
        self.sfactor = spos.z

        # length of the text of the article (ignore 1500 chars
        # that are used for menu and other stuff)
        l = max(len(self.html_string) - 1500, 0)
        # make this number a lot smaller and keep it from growing to fast
        l = sqrt(l / 10000.0)
        # diameter of the main dot
        self.diameter = self.graph.min_node_diameter + self.graph.node_diameter_factor * l

        # thickness of the ring around the node
        hidden_link_count = len(self.available_links) - self.link_count
        hidden_link_count = max(0, hidden_link_count)
        self.ring_radius = 1 + sqrt(hidden_link_count / 2.0)
        self.diameter += 6
        self.diameter += self.ring_radius

    def draw(self):
        # while loading draw grey ring around node
        d = self.diameter * self.sfactor
        if self.graph.is_loading(self):
            fill(128 / 255)
            ellipse(self.sx, self.sy, d, d)
        elif not self.html_loaded:
            fill(128 / 255)
            ellipse(self.sx, self.sy, d, d)

        col = self.node_color
        if self.graph.invert_background and self.node_color == color(0.0):
            col = color(1.0)

        # ring for outgoing links
        if self.link_count < len(self.available_links):
            d = self.diameter * self.sfactor
            push_style()
            fill(128 / 255)
            if self.graph.colorize_nodes:
                s = saturation(col)
                b = brightness(col)
                if self.graph.invert_background:
                    s *= 0.8
                    b = b * 0.5
                else:
                    s *= 0.3
                    b = 1 - (1 - b) * 0.5
                fill(hsva(hue(col), s, b))
            ellipse(self.sx, self.sy, d, d)
            pop_style()

        # white ring between center circle and link ring
        d = (self.diameter - self.ring_radius) * self.sfactor
        fill(1.0)
        if self.graph.invert_background:
            fill(0.0)
        ellipse(self.sx, self.sy, d, d)

        # main dot
        if self.html_loaded:
            d = (self.diameter - self.ring_radius - 6) * self.sfactor

            # fill(self.graph.node_color)
            push_style()
            fill(0.0)
            if self.graph.invert_background:
                fill(1.0)
            if self.graph.colorize_nodes:
                fill(col)
            ellipse(self.sx, self.sy, d, d)
            pop_style()

        # center dot for backlinks
        if self.backlink_count < len(self.available_backlinks):
            d = min((self.diameter - 6 - self.ring_radius) / 2, 4) * self.sfactor
            fill(1.0)
            if self.graph.invert_background:
                fill(0.0)
            ellipse(self.sx, self.sy, d, d)

    def draw_label(self):
        # draw text
        text_align(LEFT)
        rect_mode(CORNER)
        tfactor = 1

        # draw text for rollover_node
        if self.graph.show_text:
            if self.was_clicked or (self.graph.is_rollover(self) and self.graph.show_rollover_text):
                self.activation_time = self.graph.get_millis()

                ts = self.textsize / pow(self.graph.zoom, 0.5) * tfactor
                text_font(self.graph.font)
                text_size(ts)

                tw = text_width(self.id)
                fill(1.0, 0.8)
                if self.graph.invert_background:
                    fill(0.0, 0.8)
                rect(self.sx + (self.diameter / 2 + 4) * tfactor * self.sfactor, self.sy - (ts / 2) * tfactor, (tw + 3) * tfactor, (ts + 3) * tfactor)
                fill(80 / 255)
                if self.graph.is_rollover(self) and self.graph.show_rollover_text:
                    fill(0.0)
                    if self.graph.invert_background:
                        fill(1.0)
                text(self.id, self.sx + (self.diameter / 2 + 5) * tfactor * self.sfactor, self.sy + 6 * tfactor)
            else:
                # draw text for all nodes that are linked to the rollover node
                if self.was_clicked or self.graph.show_rollover_neighbours:
                    if self.graph.is_rollover_neighbour(self):
                        self.activation_time = self.graph.get_millis()

                dt = self.graph.get_millis() - self.activation_time
                if dt < 10000:
                    ts = self.textsize / pow(self.graph.zoom, 0.5) * tfactor
                    text_font(self.graph.font)
                    text_size(ts)

                    tw = text_width(self.id)
                    a = min(3 * (1 - dt / 10000.0), 1)
                    fill(1.0, a * 0.8)
                    if self.graph.invert_background:
                        fill(0.0, a * 0.8)
                    rect(self.sx + (self.diameter / 2 + 4) * tfactor * self.sfactor, self.sy - (ts / 2) * tfactor, (tw + 3) * tfactor, (ts + 3) * tfactor)
                    fill(80 / 255, a)
                    text(self.id, self.sx + (self.diameter / 2 + 5) * tfactor * self.sfactor, self.sy + 6 * tfactor)


class WikipediaGraph:

    def __init__(self):
        # we use a dictionary to store the nodes, because we frequently have to find them by their id,
        # which is easy to do with a dictionary
        self.node_map = {}

        self.min_click_diameter = 20

        # hovered node
        self.rollover_node = None
        # node that is dragged with the mouse
        self.selected_node = None
        # node for which loading is in progress
        self.loading_node = None
        # node that was clicked on
        self.clicked_node = None

        # we use a list because it is faster to append with new springs
        self.springs = []

        # default parameters
        self.result_count = 10

        self.spring_length = 100
        self.spring_stiffness = 0.4
        self.spring_damping = 0.9

        self.node_radius = 200
        self.node_strength = -15
        self.node_damping = 0.5

        self.invert_background = True
        self.draw_fish_eyed = True

        self.font = None
        self.textsize = 0
        self.line_weight = 3
        self.line_alpha = 100
        self.link_color = color(0.0)

        self.colorize_nodes = True
        self.min_node_diameter = 3
        self.node_diameter_factor = 1

        # lists for text analysis
        self.patterns = []
        self.colors = []

        self.show_text = True
        self.show_rollover_text = True
        self.show_rollover_neighbours = False

        self.min_x = 0
        self.min_y = 0
        self.max_x = width
        self.max_y = height

        self.auto_zoom = True
        self.zoom = 1
        self.target_zoom = 1
        # center of the boundaries of the graph
        self.center = Vec2()
        #
        self.offset = Vec2()
        self.target_offset = Vec2()

        # helpers
        self.p_millis = millis()
        # for pdf output we need to freeze time to
        # prevent text from disappearing
        self.freeze_time = False

        # new node edit
        self.editing = False
        self.edit_text = ""
        self.edit_node = None

        self.font = load_font("data/miso-regular.ttf")

        # keywords for coloing the nodes
        # scientific -> blue
        s = "model|theory|structur|component|element|concept|experiment|mathematic|chemi|device|biology|engineer|physic|scientific|science"
        self.patterns.append(re.compile(s))
        self.colors.append(color(0.0, 130 / 255, 164 / 255))

        # geography, politics -> yellow
        s = "climate|histor|planet|plant|population|ethnic|politic|atmosphere|treaty"
        self.patterns.append(re.compile(s))
        self.colors.append(color(181 / 255, 157 / 255, 0.0))

        # culture, arts -> purple
        s = "design|jazz|sculpture|culture|music|opus|period|composer|perform|literature|author|drama|poet|genre|fiction|theatre|style"
        self.patterns.append(re.compile(s))
        self.colors.append(color(89 / 255, 34 / 255, 131 / 255))

    def add_node(self, the_id, the_x, the_y):
        # check if node is already there
        find_node = self.node_map.get(the_id)

        if find_node == None:
            # create a new node
            new_node = WikipediaNode(self, the_x, the_y)

            new_node.set_id(the_id)

            new_node.set_damping(self.node_damping)
            new_node.set_strength(self.node_strength)
            new_node.set_radius(self.node_radius)

            self.node_map[the_id] = new_node

            return new_node
        else:
            return None

    def remove_node(self, the_node):
        # remove springs from/to the_node
        for i in range(len(self.springs) - 1, -1, -1):
            s = self.springs[i]
            if s.from_node == the_node or s.to_node == the_node:
                s.from_node.link_count -= 1
                self.springs.pop(i)
                s.to_node.backlink_count -= 1

        # remove the_node
        self.node_map.pop(the_node.id, None)

        # remove single nodes
        for node in list(self.node_map.values()):
            if self.get_spring_index_by_node(node) < 0 and not node.was_clicked:
                self.node_map.pop(node.id)

    def add_spring(self, from_id, to_id):
        from_node = self.node_map.get(from_id)
        to_node = self.node_map.get(to_id)

        # if one of the nodes do not exist, stop creating spring
        if from_node == None:
            return None
        if to_node == None:
            return None

        if self.get_spring(from_node, to_node) == None:
            # create a new spring
            new_spring = Spring(from_node, to_node, self.spring_length, self.spring_stiffness, 0.9)

            from_node.link_count += 1
            to_node.backlink_count += 1

            self.springs.append(new_spring)
            return new_spring

        return None

    def get_node_by_id(self, the_id):
        node = self.node_map.get(the_id)
        return node

    def get_node_by_screen_pos(self, the_x, the_y):
        mx = (the_x - width / 2) / self.zoom - self.offset.x
        my = (the_y - height / 2) / self.zoom - self.offset.y

        return self.get_node_by_pos(mx, my)

    def get_node_by_pos(self, the_x, the_y):
        selected_node = None
        for check_node in self.node_map.values():
            d = dist(the_x, the_y, check_node.sx, check_node.sy)
            if d < max(check_node.diameter / 2, self.min_click_diameter):
                selected_node = check_node
        return selected_node

    def get_spring_index_by_node(self, the_node):
        for i in range(len(self.springs)):
            s = self.springs[i]
            if s.from_node == the_node or s.to_node == the_node:
                return i
        return -1

    def get_spring(self, the_from_node, the_to_node):
        for i in range(len(self.springs)):
            s = self.springs[i]
            if s.from_node == the_from_node and s.to_node == the_to_node:
                return s
        return None

    def get_zoom(self):
        return self.target_zoom

    def set_zoom(self, the_zoom):
        self.target_zoom = the_zoom

    def get_offset(self):
        return Vec2(self.offset.x, self.offset.y)

    def set_offset(self, the_offset_x, the_offset_y):
        self.offset.x = the_offset_x
        self.offset.y = the_offset_y
        self.target_offset.x = self.offset.x
        self.target_offset.y = self.offset.y

    def get_loading_node(self):
        return self.loading_node

    def set_loading_node(self, the_node):
        self.loading_node = the_node

    def is_loading(self, the_node):
        if the_node == self.loading_node:
            return True
        return False

    def is_rollover(self, the_node):
        if the_node == self.rollover_node:
            return True
        return False

    def is_rollover_neighbour(self, the_node):
        if self.get_spring(the_node, self.rollover_node) != None:
            return True
        if self.get_spring(self.rollover_node, the_node) != None:
            return True
        return False

    def is_selected(self, the_node):
        if the_node == self.selected_node:
            return True
        return False

    def get_millis(self):
        if self.freeze_time:
            return self.p_millis
        return millis()

    def set_result_count(self, the_result_count):
        self.result_count = the_result_count

    def set_spring_length(self, the_length):
        if the_length != self.spring_length:
            self.spring_length = the_length
            for i in range(len(self.springs)):
                s = self.springs[i]
                s.set_length(self.spring_length)

    def set_spring_stiffness(self, the_stiffness):
        if the_stiffness != self.spring_stiffness:
            self.spring_stiffness = the_stiffness
            for i in range(len(self.springs)):
                s = self.springs[i]
                s.set_stiffness(self.spring_stiffness)

    def set_spring_damping(self, the_damping):
        if the_damping != self.spring_damping:
            self.spring_damping = the_damping
            for i in range(len(self.springs)):
                s = self.springs[i]
                s.set_damping(self.spring_damping)

    def set_node_radius(self, the_radius):
        if the_radius != self.node_radius:
            self.node_radius = the_radius
            for node in self.node_map.values():
                node.set_radius(self.node_radius)

    def set_node_strength(self, the_strength):
        if the_strength != self.node_strength:
            self.node_strength = the_strength
            for node in self.node_map.values():
                node.set_strength(self.node_strength)

    def set_node_damping(self, the_damping):
        if the_damping != self.node_damping:
            self.node_damping = the_damping
            for node in self.node_map.values():
                node.set_damping(self.node_damping)

    def set_invert_background(self, the_invert_background):
        self.invert_background = the_invert_background

    def set_draw_hyperbolic(self, the_draw_hyperbolic):
        self.draw_fish_eyed = the_draw_hyperbolic

    def set_line_weight(self, the_line_weight):
        self.line_weight = the_line_weight

    def set_line_alpha(self, the_line_alpha):
        self.line_alpha = the_line_alpha

    def set_colorize_nodes(self, the_value):
        self.colorize_nodes = the_value

    def set_min_node_diameter(self, the_value):
        self.min_node_diameter = the_value

    def set_node_diameter_factor(self, the_value):
        self.node_diameter_factor = the_value

    def set_link_color(self, the_link_color):
        self.link_color = the_link_color

    def set_show_text(self, the_show_text):
        self.show_text = the_show_text

    def set_show_rollover_text(self, the_show_rollover_text):
        self.show_rollover_text = the_show_rollover_text

    def set_show_rollover_neighbours(self, the_show_rollover_neighbours):
        self.show_rollover_neighbours = the_show_rollover_neighbours

    def screen_pos(self, the_pos):
        if self.draw_fish_eyed:
            # position of the node transformed, so that x and y is 0
            # if the node is in the center of the screen
            x = the_pos.x + self.offset.x / self.zoom
            y = the_pos.y + self.offset.y / self.zoom
            # get polar coordinates of the point
            distance = sqrt(x * x + y * y)
            angle = atan2(y, x)

            # fisheye projection
            radius = min(width, height) / 2
            dist_angle = atan(distance / (radius / 2) * self.zoom) / HALF_PI
            new_distance = dist_angle * radius / self.zoom

            # transform polar coordinates back into cartesian coordinates
            # new position
            new_x = new_distance * cos(angle) - self.offset.x
            new_y = new_distance * sin(angle) - self.offset.y
            new_scale = min(1.2 - dist_angle, 1)
            return Vec3(new_x, new_y, new_scale)
        else:
            return Vec3(the_pos.x, the_pos.y, 1)

    def local_to_global(self, the_x, the_y):
        mx = (the_x + self.offset.x) * self.zoom + width / 2
        my = (the_y + self.offset.y) * self.zoom + height / 2

        return Vec2(mx, my)

    def global_to_local(self, the_x, the_y):
        mx = (the_x - width / 2) / self.zoom - self.offset.x
        my = (the_y - height / 2) / self.zoom - self.offset.y

        return Vec2(mx, my)

    def to_string(self):
        s = ""

        for node in self.node_map.values():
            s += "[ " + str(node.x) + ", " + str(node.y) + ", " + str(node.z) + " ]\n"
        return s

    def update(self):
        # use this function also to get actual width and heigth of the graph
        self.min_x = float("inf")
        self.min_y = float("inf")
        self.max_x = -float("inf")
        self.max_y = -float("inf")

        # make a list out of the values in node_map
        nodes = list(self.node_map.values())

        for i in range(len(nodes)):
            nodes[i].attract_nodes(nodes)
        for i in range(len(self.springs)):
            s = self.springs[i]
            if s == None:
                break
            s.update()
        for i in range(len(nodes)):
            nodes[i].update()

            self.min_x = min(nodes[i].x, self.min_x)
            self.max_x = max(nodes[i].x, self.max_x)
            self.min_y = min(nodes[i].y, self.min_y)
            self.max_y = max(nodes[i].y, self.max_y)

        if self.selected_node != None:
            # when dragging a node
            self.selected_node.x = (mouse_x - width / 2) / self.zoom - self.offset.x
            self.selected_node.y = (mouse_y - height / 2) / self.zoom - self.offset.y
        elif self.auto_zoom:
            # otherwise recalc zoom, center and offset
            self.center = Vec2((self.min_x + self.max_x) / 2, (self.min_y + self.max_y) / 2)
            self.target_offset = Vec2(-self.center.x, -self.center.y)

            dx = self.max_x - self.min_x
            dy = self.max_y - self.min_y
            qx = 100
            qy = 100
            if dx > 0:
                qx = (width - 50) / dx
            if dy > 0:
                qy = (height - 50) / dy
            self.target_zoom = min(min(qx, qy), 1)

        # check if there is a node hovered
        self.rollover_node = self.get_node_by_screen_pos(mouse_x, mouse_y)

    def draw(self):
        dt = 0
        if not self.freeze_time:
            m = millis()
            dt = m - self.p_millis
            self.p_millis = m

        # smooth movement of canvas
        accomplish_per_second = 0.95
        f = pow(1 / (1 - accomplish_per_second), -dt / 1000.0)

        d = self.target_offset - self.offset
        d.mult(f)
        self.offset = self.target_offset - d

        self.zoom = self.target_zoom - ((self.target_zoom - self.zoom) * f)

        push_style()

        push_matrix()
        translate(width / 2, height / 2)
        scale(self.zoom)
        translate(self.offset.x, self.offset.y)

        for node in self.node_map.values():
            node.loader_loop()

        # draw origin
        # fill(1.0, 0.0, 0.0)
        # ellipse(0, 0, 4, 4)

        # draw springs
        for i in range(len(self.springs)):
            s = self.springs[i]
            if s == None:
                break
            stroke(hsva(hue(self.link_color), saturation(self.link_color), brightness(self.link_color), self.line_alpha / 100))
            stroke_weight(self.line_weight)
            self.draw_arrow(s.from_node, s.to_node)
            no_stroke()

        # draw nodes
        for node in self.node_map.values():
            node.draw()

        # draw node labels
        for node in self.node_map.values():
            node.draw_label()

        pop_matrix()

        # the name of the new node while it is typed (a text field in the original)
        if self.editing:
            sp = self.local_to_global(self.edit_node.sx, self.edit_node.sy)
            fill(1.0)
            rect(int(sp.x + 10), int(sp.y - 10), 150, 20)
            fill(0.0)
            text_font(self.font)
            text_size(16)
            text(self.edit_text, int(sp.x + 14), int(sp.y + 5))

        pop_style()

    def draw_arrow(self, n1, n2):

        d = Vec2(n2.sx - n1.sx, n2.sy - n1.sy)
        margin1 = n1.diameter * n1.sfactor / 2.0 + 3 + self.line_weight / 2
        margin2 = n2.diameter * n2.sfactor / 2.0 + 3 + self.line_weight / 2

        if d.mag() > margin1 + margin2:
            d = d.normalize()
            line(n1.sx + d.x * margin1, n1.sy + d.y * margin1, n2.sx - d.x * margin2, n2.sy - d.y * margin2)

            a = atan2(d.y, d.x)
            push_matrix()
            translate(n2.sx - d.x * margin2, n2.sy - d.y * margin2)
            rotate(a)
            l = 1 + self.line_weight
            line(0, 0, -l, -l)
            line(0, 0, -l, l)
            pop_matrix()

    def get_width(self):
        return 1

    def encode_url(self, name):
        sb = ""
        utf8 = name.encode("utf-8")
        for i in range(len(utf8)):
            value = utf8[i] & 0xff
            if value < 33 or value > 126:
                sb += "%"
                sb += "%02X" % value
            else:
                sb += chr(value)
        return sb

    def text_to_color(self, the_text):
        len_ = len(self.patterns)
        counters = [0] * len_
        counter_sums = [0] * len_

        for i in range(len_):
            m = self.patterns[i].findall(the_text.lower())
            counters[i] = len(m)

        # sumarize counters
        counter_sums[0] = counters[0]
        for i in range(1, len_):
            counter_sums[i] = counter_sums[i - 1] + counters[i]
        # if all counters are 0, return black
        if counter_sums[len_ - 1] == 0:
            return color(0.0)

        # interpolate colors
        result = self.colors[0]
        for i in range(1, len_):
            amount = counters[i] / float(counter_sums[i])
            result = lerp_color(result, self.colors[i], amount)
        return result

    def mouse_pressed(self):
        self.clicked_node = self.get_node_by_screen_pos(mouse_x, mouse_y)

        if self.clicked_node != None:
            if mouse_button == RIGHT:
                self.selected_node = self.clicked_node

                # double click right -> open page in browser
                if mouse_event.get_click_count() == 2:  # GAP: no click count on mouse events (mouseEvent.getClickCount())
                    link("http://en.wikipedia.org/wiki/" + self.encode_url(self.clicked_node.id))  # GAP: link(url) (open a URL in the browser) is missing

                return True

        return False

    def mouse_released(self):
        if self.selected_node != None:
            self.selected_node = None

        if self.clicked_node != None:

            if last_mouse_button == LEFT:

                if key_is_pressed and key_code == SHIFT:
                    # delete clicked node
                    if self.clicked_node == self.edit_node:
                        self.editing = False

                    self.remove_node(self.clicked_node)
                    return True
                elif not key_is_pressed:
                    self.clicked_node.was_clicked = True
                    # load links from clicked node
                    if self.clicked_node.available_links_loaded:
                        added_nodes = 0
                        for i in range(len(self.clicked_node.available_links)):
                            if added_nodes >= self.result_count:
                                break

                            title = self.clicked_node.available_links[i]

                            added_node = self.add_node(title, self.clicked_node.x + random(-5, 5), self.clicked_node.y + random(-5, 5))
                            if added_node != None:
                                added_nodes += 1

                            added_spring = self.add_spring(self.clicked_node.id, title)
                    return True
                elif key_is_pressed and key_code == ALT:
                    self.clicked_node.was_clicked = True
                    # load links to clicked node
                    if self.clicked_node.available_backlinks_loaded:

                        added_nodes = 0
                        for i in range(len(self.clicked_node.available_backlinks)):
                            if added_nodes >= self.result_count:
                                break

                            title = self.clicked_node.available_backlinks[i]

                            # if my_wikipedia_graph.get_node_by_id(title) == None:
                            added_node = self.add_node(title, self.clicked_node.x + random(-5, 5), self.clicked_node.y + random(-5, 5))
                            if added_node != None:
                                added_nodes += 1

                            added_spring = self.add_spring(title, self.clicked_node.id)
                    return True
        else:
            # doubleclick on canvas
            if last_mouse_button == LEFT:
                if mouse_event.get_click_count() == 2:  # GAP: no click count on mouse events (mouseEvent.getClickCount())
                    if not self.editing:
                        # create new node
                        self.editing = True

                        p = self.global_to_local(mouse_x, mouse_y)
                        self.edit_node = self.add_node("", p.x, p.y)

                        self.edit_text = ""

                        return True

        return False

    def confirm_edit(self, the_text):
        self.editing = False

        px = self.edit_node.x
        py = self.edit_node.y
        self.remove_node(self.edit_node)
        new_node = self.add_node(the_text, px, py)
        new_node.was_clicked = True


# ------ initial parameters and declarations ------

# start with this keyword
act_keyword = "Design"
act_node = None

zoom = 0.75  # slider 0.05 - 1
auto_zoom = False  # toggle

result_count = 10  # slider 1 - 50

spring_length = 100  # slider 10 - 500
spring_stiffness = 0.4  # slider 0 - 1
spring_damping = 0.9  # slider 0 - 1

node_radius = 200  # slider 0 - 500
node_strength = 15  # slider 0 - 50
node_damping = 0.5  # slider 0 - 1

invert_background = False  # toggle
draw_fish_eyed = False  # toggle
line_weight = 1  # slider 1 - 20
line_alpha = 100  # slider 0 - 100
link_color = 0.89  # slider 0 - 1

colorize_nodes = True  # toggle
min_node_diameter = 8  # slider 0 - 60
node_diameter_factor = 20  # slider 0 - 100
node_color = 1

show_text = True  # toggle
show_rollover_text = True
show_rollover_neighbours = False  # toggle

# the graph
my_wikipedia_graph = None

# ------ mouse interaction ------

dragging = False
offset_x = 0
offset_y = 0
click_x = 0
click_y = 0
click_offset_x = 0
click_offset_y = 0
last_mouse_button = 0

# ------ image output ------

save_one_frame = False


def setup():
    global my_wikipedia_graph, act_node
    size(800, 800)

    no_stroke()

    # make window resizable
    window_resizable(True)

    # construct a new WikipediaGraph
    my_wikipedia_graph = WikipediaGraph()

    # add the first node to the graph
    act_node = my_wikipedia_graph.add_node(act_keyword, 0, 0)
    act_node.was_clicked = True


def draw():
    global zoom, save_one_frame

    # ------ white/black background ------

    bg_color = color(1.0)
    if invert_background:
        bg_color = color(0.0)
    background(bg_color)

    # ------ update zooming and position of graph ------
    my_wikipedia_graph.auto_zoom = auto_zoom
    if auto_zoom:
        zoom = my_wikipedia_graph.get_zoom()
    else:
        my_wikipedia_graph.set_zoom(zoom)
        # canvas dragging
        if dragging:
            my_wikipedia_graph.set_offset(click_offset_x + (mouse_x - click_x), click_offset_y + (mouse_y - click_y))

    # ------ update and draw graph ------
    my_wikipedia_graph.set_result_count(result_count)

    my_wikipedia_graph.set_spring_length(spring_length)
    my_wikipedia_graph.set_spring_stiffness(spring_stiffness)
    my_wikipedia_graph.set_spring_damping(spring_damping)

    my_wikipedia_graph.set_node_radius(node_radius)
    my_wikipedia_graph.set_node_strength(-node_strength)
    my_wikipedia_graph.set_node_damping(node_damping)

    my_wikipedia_graph.set_invert_background(invert_background)
    my_wikipedia_graph.set_draw_hyperbolic(draw_fish_eyed)

    my_wikipedia_graph.set_line_weight(line_weight)
    my_wikipedia_graph.set_line_alpha(line_alpha)
    my_wikipedia_graph.set_link_color(linear_color(link_color))

    my_wikipedia_graph.set_min_node_diameter(min_node_diameter)
    my_wikipedia_graph.set_node_diameter_factor(node_diameter_factor)
    my_wikipedia_graph.set_colorize_nodes(colorize_nodes)

    my_wikipedia_graph.set_show_text(show_text)
    my_wikipedia_graph.set_show_rollover_text(show_rollover_text)
    my_wikipedia_graph.set_show_rollover_neighbours(show_rollover_neighbours)

    my_wikipedia_graph.update()
    my_wikipedia_graph.draw()

    # ------ image output ------
    if save_one_frame == True:
        save_frame(timestamp() + ".png")
        save_one_frame = False


# converts special chars in a string to be suitable for a url
def encode_url(url):
    return urllib.parse.quote(url, safe=":/?&=")


# function to make a broad range of colors available by
# a single number between 0 and 1
def linear_color(the_value):
    h = (the_value) * 360
    s = abs(sin((1 - the_value) * PI * 9)) * 60 + (1 - the_value) * 40
    b = abs(cos((1 - the_value) * PI * 4.5)) * 60 + (the_value) * 40
    new_color = hsva(h, s / 100, b / 100)

    return new_color


# ------ key and mouse events ------

def key_pressed():
    global colorize_nodes, draw_fish_eyed, zoom, save_one_frame
    if not my_wikipedia_graph.editing:
        if key == "1":
            colorize_nodes = not colorize_nodes
        if key == "2":
            draw_fish_eyed = not draw_fish_eyed

        if key_code == UP:
            zoom += 0.05
        if key_code == DOWN:
            zoom -= 0.05
        zoom = max(zoom, 0.1)

        if key == "s" or key == "S":
            save_one_frame = True
    else:
        # type the name of the new node, enter confirms it
        if key_code == ENTER:
            my_wikipedia_graph.confirm_edit(my_wikipedia_graph.edit_text)
        elif key_code == BACKSPACE:
            my_wikipedia_graph.edit_text = my_wikipedia_graph.edit_text[:-1]
        elif key != None and len(key) == 1:
            my_wikipedia_graph.edit_text += key


def mouse_pressed():
    global last_mouse_button, dragging, click_x, click_y, click_offset_x, click_offset_y
    last_mouse_button = mouse_button

    # tell graph that mouse was pressed
    event_handled = my_wikipedia_graph.mouse_pressed()
    # if the graph didn't do anything with the mouse event
    if not event_handled:
        # canvas dragging
        if mouse_button == RIGHT:
            dragging = True
            click_x = mouse_x
            click_y = mouse_y
            click_offset_x = my_wikipedia_graph.offset.x
            click_offset_y = my_wikipedia_graph.offset.y


def mouse_released():
    global dragging
    event_handled = my_wikipedia_graph.mouse_released()
    dragging = False


def mouse_entered():  # GAP: no mouse_entered() callback
    loop()


def mouse_exited():  # GAP: no mouse_exited() callback
    no_loop()


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
