# M_4_4_01_TOOL
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

# play around with a 3d grid of nodes and draw them in different ways
#
# MOUSE
# left click          : apply attractor force (repulsion)
# shift + left click  : apply attractor force (attraction)
# right click + drag  : rotate
#
# KEYS
# arrow keys          : rotate
# r                   : reset
# s                   : save png
#
# The controlP5 panel of the original is not ported; its sliders and toggles
# are the variables below (reset() sets them to the start values), and its
# preset buttons are the functions reset(), set1(), set2(), set3() and set4(),
# which can be called at the end of setup().
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


# Attractor class of the generativedesign library
class Attractor:

    # attractor modes
    BASIC = 0
    SMOOTH = 1
    TWIRL = 2

    def __init__(self, the_x=0, the_y=0, the_z=0):
        # position
        self.x = the_x
        self.y = the_y
        self.z = the_z

        self.mode = Attractor.BASIC

        # radius of impact
        self.radius = 200
        # strength: positive for attraction, negative for repulsion
        self.strength = 1
        # parameter that influences the form of the function
        self.ramp = 0.5  # 0.01 - 0.99

    # ------ rotate position around origin ------
    def rotate_x(self, the_angle):
        newy = self.y * cos(the_angle) - self.z * sin(the_angle)
        newz = self.y * sin(the_angle) + self.z * cos(the_angle)
        self.y = newy
        self.z = newz

    def rotate_y(self, the_angle):
        newx = self.x * cos(-the_angle) - self.z * sin(-the_angle)
        newz = self.x * sin(-the_angle) + self.z * cos(-the_angle)
        self.x = newx
        self.z = newz

    # ------ calculate attraction ------
    def attract(self, the_node):
        # calculate distance
        dx = self.x - the_node.x
        dy = self.y - the_node.y
        dz = self.z - the_node.z
        d = mag(dx, dy, dz)
        f = 0

        if self.mode == Attractor.BASIC:
            if d > 0 and d < self.radius:
                # calculate force
                s = d / self.radius
                f = (1 / pow(s, 0.5 * self.ramp) - 1)
                f = self.strength * f / self.radius
        else:
            if d > 0 and d < self.radius:
                # calculate force
                s = pow(d / self.radius, 1 / self.ramp)
                f = s * 9 * self.strength * (1 / (s + 1) + ((s - 3) / 4)) / d

        # apply force to node velocity
        if self.mode != Attractor.TWIRL:
            the_node.velocity.x += dx * f
            the_node.velocity.y += dy * f
            the_node.velocity.z += dz * f
        else:
            # swapping of dx and dy makes the twirl
            the_node.velocity.x += dy * f
            the_node.velocity.y -= dx * f
            the_node.velocity.z += dz * f

    # ------ getters and setters ------
    def set_mode(self, the_mode):
        self.mode = the_mode


# ------ initial parameters and declarations ------

default_colors = [color(0.0, 130 / 255, 164 / 255)]

colors = None

freeze = False

max_count = 50
x_count = 0  # slider 0 - max_count
y_count = 0  # slider 0 - max_count
z_count = 0  # slider 0 - max_count
old_x_count = 0
old_y_count = 0
old_z_count = 0
grid_step_x = 0  # slider 1 - 50
grid_step_y = 0  # slider 1 - 50
grid_step_z = 0  # slider 1 - 50
old_grid_step_x = 0
old_grid_step_y = 0
old_grid_step_z = 0

attractor_radius = 150  # slider 1 - 1000
attractor_strength = 3  # slider 0 - 10
attractor_ramp = 1  # slider 0.1 - 3
node_damping = 0.1  # slider 0 - 1

draw_x = True  # toggle
draw_y = True  # toggle
draw_z = True  # toggle
lock_x = False  # toggle
lock_y = False  # toggle
lock_z = False  # toggle

invert_background = False  # toggle
line_weight = 1  # slider 0.2 - 20
line_alpha = 50  # slider 0 - 100

draw_lines = True  # toggle
draw_curves = False  # toggle

# nodes array
nodes = []
for iz in range(max_count * 2 + 1):
    layer = []
    for iy in range(max_count * 2 + 1):
        layer.append([None] * (max_count * 2 + 1))
    nodes.append(layer)

# attraktor
my_attractor = None

# ------ mouse interaction ------

offset_x = 0
offset_y = 0
click_x = 0
click_y = 0
rotation_x = 0
rotation_y = 0
target_rotation_x = 0
target_rotation_y = 0
click_rotation_x = 0
click_rotation_y = 0
mouse_in_window = False

# ------ image output ------

save_one_frame = False


def setup():
    global my_attractor
    size(1000, 1000)
    mode_3d()
    hint(DISABLE_DEPTH_TEST)  # GAP: hint(DISABLE_DEPTH_TEST) (draw without depth testing) is missing

    # init attractor
    my_attractor = Attractor()
    my_attractor.set_mode(Attractor.SMOOTH)

    # init grid
    reset()


def draw():
    global offset_x, offset_y, target_rotation_x, target_rotation_y, rotation_x, rotation_y
    global old_x_count, old_y_count, old_z_count, save_one_frame

    bg_color = color(1.0)
    circle_color = color(0.0)
    if invert_background:
        bg_color = color(0.0)
        circle_color = color(1.0)
    background(bg_color)

    no_stroke()
    fill(bg_color)
    rect(-width / 2, -height / 2, width, height)

    no_fill()
    stroke(0.0, 1.0)
    stroke_weight(line_weight)

    # ------ set view ------
    push_matrix()

    if mouse_is_pressed and mouse_button == RIGHT:
        offset_x = (mouse_x - click_x)
        offset_y = -(mouse_y - click_y)

        target_rotation_x = min(max(click_rotation_x + offset_y / float(width) * TWO_PI, -HALF_PI), HALF_PI)
        target_rotation_y = click_rotation_y + offset_x / float(height) * TWO_PI
    rotation_x += (target_rotation_x - rotation_x) * 0.25
    rotation_y += (target_rotation_y - rotation_y) * 0.25
    rotate_x(rotation_x)
    rotate_y(rotation_y)

    # ------ set parameters ------
    if x_count != old_x_count or y_count != old_y_count or z_count != old_z_count:
        # init_grid()
        old_x_count = x_count
        old_y_count = y_count
        old_z_count = z_count

    if nodes[0][0][0].damping != node_damping:
        update_damping()

    my_attractor.radius = attractor_radius
    my_attractor.ramp = attractor_ramp
    if mouse_is_pressed and mouse_button == LEFT:
        if not key_is_pressed:
            # attraction, if left click
            my_attractor.strength = -attractor_strength
        elif key_is_pressed and key_code == SHIFT:
            # repulsion, if shift + left click
            my_attractor.strength = attractor_strength
    else:
        # otherwise no attraction or repulsion
        my_attractor.strength = 0

    random_seed(433)
    m = millis() / 1000.0

    # set attractor at the mouse position
    my_attractor.x = (mouse_x - width / 2)
    my_attractor.y = (mouse_y - height / 2)
    my_attractor.z = 0
    my_attractor.rotate_x(-rotation_x)
    my_attractor.rotate_y(-rotation_y)

    # ------ attract and update node positions ------
    for iz in range(max_count - z_count, max_count + z_count + 1):
        for iy in range(max_count - y_count, max_count + y_count + 1):
            for ix in range(max_count - x_count, max_count + x_count + 1):
                my_attractor.attract(nodes[iz][iy][ix])
                nodes[iz][iy][ix].update(lock_x, lock_y, lock_z)

    # ------ draw lines ------
    line_drawn = False

    # x
    if draw_x and x_count > 0:
        for iz in range(max_count - z_count, max_count + z_count + 1):
            c = colors[iz % len(colors)]
            if c == color(0.0) and invert_background:
                c = color(1.0)
            stroke(c.with_alpha(line_alpha / 100))
            for iy in range(max_count - y_count, max_count + y_count + 1):
                draw_line(nodes[iz][iy], x_count, draw_curves)
        line_drawn = True

    # y
    if draw_y and y_count > 0:
        for iz in range(max_count - z_count, max_count + z_count + 1):
            c = colors[iz % len(colors)]
            if c == color(0.0) and invert_background:
                c = color(1.0)
            stroke(c.with_alpha(line_alpha / 100))
            for ix in range(max_count - x_count, max_count + x_count + 1):
                pts = [None] * (max_count * 2 + 1)
                ii = 0
                for iy in range(max_count * 2 + 1):
                    pts[ii] = nodes[iz][iy][ix]
                    ii += 1
                draw_line(pts, y_count, draw_curves)
        line_drawn = True

    # z
    if draw_z and z_count > 0:
        for iy in range(max_count - y_count, max_count + y_count + 1):
            c = colors[iy % len(colors)]
            if c == color(0.0) and invert_background:
                c = color(1.0)
            stroke(c.with_alpha(line_alpha / 100))
            for ix in range(max_count - x_count, max_count + x_count + 1):
                pts = [None] * (max_count * 2 + 1)
                ii = 0
                for iz in range(max_count * 2 + 1):
                    pts[ii] = nodes[iz][iy][ix]
                    ii += 1
                draw_line(pts, z_count, draw_curves)
        line_drawn = True

    # if no lines were drawn, draw dots
    if not line_drawn:
        for iz in range(max_count - z_count, max_count + z_count + 1):
            c = colors[iz % len(colors)]
            stroke(red(c), green(c), blue(c), line_alpha / 100)

            for iy in range(max_count - y_count, max_count + y_count + 1):
                for ix in range(max_count - x_count, max_count + x_count + 1):
                    n = nodes[iz][iy][ix]
                    point(n.x, n.y, n.z)  # GAP: 3D point(x, y, z)
        pop_matrix()
    else:
        pop_matrix()

    # ------ draw attractor radius ------

    if mouse_in_window and not save_one_frame:
        no_fill()
        stroke_weight(1)
        stroke(circle_color.with_alpha(0.2))
        ellipse(mouse_x - width / 2, mouse_y - height / 2, my_attractor.radius * 2, my_attractor.radius * 2)

    # ------ image output ------

    if save_one_frame:
        save_frame(timestamp() + ".png")
        save_one_frame = False


def draw_line(points, len_, curves):
    # this funktion draws a line from an array of nodes
    # len_   : number of points to each side of the center index of the array
    #          example: array-length=21, len_=5 -> points[5] to points[15] will be drawn
    # curves : if true, points will be connected with curves (a bit like curve_vertex,
    #          not as accurate, but faster)

    d1 = Vec3()
    d2 = Vec3()

    # first and last index to be drawn
    i1 = (len(points) - 1) // 2 - len_
    i2 = (len(points) - 1) // 2 + len_

    # draw first point
    begin_shape()
    vertex(points[i1].x, points[i1].y, points[i1].z)  # GAP: vertex(x, y, z) in immediate mode
    q0 = 0.5

    for i in range(i1 + 1, i2 + 1):
        if curves:
            if i < i2:
                # distance to previous and next point
                l1 = dist(points[i].x, points[i].y, points[i].z, points[i - 1].x, points[i - 1].y, points[i - 1].z)
                l2 = dist(points[i].x, points[i].y, points[i].z, points[i + 1].x, points[i + 1].y, points[i + 1].z)
                # vector form previous to next point
                d2 = Vec3(points[i + 1].x - points[i - 1].x, points[i + 1].y - points[i - 1].y, points[i + 1].z - points[i - 1].z)
                # shortening of this vector
                d2.mult(0.333)
                # how to distribute d2 to the anchors
                q1 = l1 / (l1 + l2)
                q2 = l2 / (l1 + l2)
            else:
                # special handling for the last index
                l1 = dist(points[i].x, points[i].y, points[i].z, points[i - 1].x, points[i - 1].y, points[i - 1].z)
                l2 = 0
                d2 = Vec3(0, 0, 0)
                q1 = l1 / (l1 + l2)
                q2 = 0
            # draw bezier_vertex
            bezier_vertex(points[i - 1].x + d1.x * q0, points[i - 1].y + d1.y * q0, points[i - 1].z + d1.z * q0,
                          points[i].x - d2.x * q1, points[i].y - d2.y * q1, points[i].z - d2.z * q1,
                          points[i].x, points[i].y, points[i].z)  # GAP: 3D bezier_vertex(cx1, cy1, cz1, cx2, cy2, cz2, x, y, z)
            # remember d2 and q2 for the next iteration
            d1 = d2.copy()
            q0 = q2
        else:
            vertex(points[i].x, points[i].y, points[i].z)  # GAP: vertex(x, y, z) in immediate mode

    end_shape()


def init_grid():
    for iz in range(max_count * 2 + 1):
        for iy in range(max_count * 2 + 1):
            for ix in range(max_count * 2 + 1):
                x_pos = (ix - max_count) * grid_step_x
                y_pos = (iy - max_count) * grid_step_y
                z_pos = (iz - max_count) * grid_step_z

                if nodes[iz][iy][ix] == None:
                    n = Node(x_pos, y_pos, z_pos)
                    n.min_x = -20000
                    n.max_x = 20000
                    n.min_y = -20000
                    n.max_y = 20000
                    n.min_z = -20000
                    n.max_z = 20000
                else:
                    n = nodes[iz][iy][ix]
                    n.x = x_pos
                    n.y = y_pos
                    n.z = z_pos
                    n.velocity.x = 0
                    n.velocity.y = 0
                    n.velocity.z = 0

                n.damping = node_damping

                nodes[iz][iy][ix] = n


def scale_grid(the_factor_x, the_factor_y, the_factor_z):
    for iz in range(9):
        for iy in range(max_count * 2 + 1):
            for ix in range(max_count * 2 + 1):
                nodes[iz][iy][ix].x *= the_factor_x
                nodes[iz][iy][ix].y *= the_factor_y
                nodes[iz][iy][ix].z *= the_factor_z


def update_damping():
    for iz in range(max_count * 2 + 1):
        for iy in range(max_count * 2 + 1):
            for ix in range(max_count * 2 + 1):
                nodes[iz][iy][ix].damping = node_damping


# ------ functions for reset and presets ------

def reset():
    global colors, target_rotation_x, target_rotation_y
    colors = default_colors
    set_paras(10, 10, 10, 30, 30, 30, 150, 3, 1, 0.1, False, 1, 50, True, True, True, False, False, False, False)

    init_grid()

    target_rotation_x = 0
    target_rotation_y = 0


def set1():
    global colors, target_rotation_x, target_rotation_y
    colors = [None] * 1
    colors[0] = color(0.0)

    set_paras(10, 50, 10, 20, 10, 20, 400, 1, 1, 0.03, False, 6, 10, False, True, False, False, False, False, False)
    init_grid()

    target_rotation_x = 0
    target_rotation_y = 0


def set2():
    global colors, target_rotation_x, target_rotation_y
    colors = [None] * 2
    colors[0] = hsva(52, 1.0, 0.71, 1.0)
    colors[1] = hsva(192, 1.0, 0.64, 1.0)

    set_paras(20, 20, 2, 20, 10, 40, 200, 3, 1, 0.1, False, 1, 50, True, True, False, True, True, False, True)
    init_grid()
    target_rotation_x = -HALF_PI
    target_rotation_y = 0


def set3():
    global colors, target_rotation_x, target_rotation_y
    colors = [None] * 1
    colors[0] = hsva(323, 1.0, 0.77)

    set_paras(20, 4, 20, 4, 15, 25, 300, 8, 1, 0.1, True, 1, 50, False, True, False, True, False, True, False)
    init_grid()

    target_rotation_x = 0
    target_rotation_y = -HALF_PI


def set4():
    global colors, target_rotation_x, target_rotation_y
    m = max_count * 2 + 1
    colors = [None] * m
    for i in range(m):
        h = remap(i, 0, m, 100, 300)
        colors[i] = hsva(h, 0.6, 0.5)

    set_paras(10, 10, 50, 30, 30, 6, 300, 3, 1, 0.1, False, 1, 50, True, False, False, False, False, False, False)
    init_grid()

    target_rotation_x = 0
    target_rotation_y = 0


def set_paras(the_x_count, the_y_count, the_z_count, the_grid_step_x, the_grid_step_y, the_grid_step_z,
              the_attractor_radius, the_attractor_strength, the_attractor_ramp, the_node_damping,
              the_invert_background, the_line_weigth, the_line_alpha,
              the_draw_x, the_draw_y, the_draw_z,
              the_lock_x, the_lock_y, the_lock_z, the_draw_curves):
    global x_count, y_count, z_count, old_x_count, old_y_count, old_z_count
    global grid_step_x, grid_step_y, grid_step_z, old_grid_step_x, old_grid_step_y, old_grid_step_z
    global attractor_radius, attractor_strength, attractor_ramp, node_damping
    global invert_background, line_weight, line_alpha
    global draw_x, draw_y, draw_z, lock_x, lock_y, lock_z, draw_lines, draw_curves

    x_count = the_x_count
    y_count = the_y_count
    z_count = the_z_count
    old_x_count = the_x_count
    old_y_count = the_y_count
    old_z_count = the_z_count

    # a new grid step scales the existing grid
    grid_step_x = the_grid_step_x
    if old_grid_step_x != 0:
        scale_grid(grid_step_x / old_grid_step_x, 1, 1)
    old_grid_step_x = grid_step_x
    grid_step_y = the_grid_step_y
    if old_grid_step_y != 0:
        scale_grid(1, grid_step_y / old_grid_step_y, 1)
    old_grid_step_y = grid_step_y
    grid_step_z = the_grid_step_z
    if old_grid_step_z != 0:
        scale_grid(1, 1, grid_step_z / old_grid_step_z)
    old_grid_step_z = grid_step_z

    attractor_radius = the_attractor_radius
    attractor_strength = the_attractor_strength
    attractor_ramp = the_attractor_ramp
    node_damping = the_node_damping

    invert_background = the_invert_background
    line_weight = the_line_weigth
    line_alpha = the_line_alpha
    draw_x = the_draw_x
    draw_y = the_draw_y
    draw_z = the_draw_z

    lock_x = the_lock_x
    lock_y = the_lock_y
    lock_z = the_lock_z

    the_draw_lines = not the_draw_curves
    draw_lines = the_draw_lines
    draw_curves = the_draw_curves


def key_pressed():
    global target_rotation_x, target_rotation_y, save_one_frame, freeze

    if key_code == LEFT_ARROW:
        target_rotation_y += 0.02
    if key_code == RIGHT_ARROW:
        target_rotation_y -= 0.02
    if key_code == UP:
        target_rotation_x -= 0.02
    if key_code == DOWN:
        target_rotation_x += 0.02

    if key == "r" or key == "R":
        # reset()
        init_grid()
    if key == "s" or key == "S":
        save_one_frame = True

    if key == " ":
        freeze = not freeze
        if freeze:
            no_loop()
        else:
            loop()


def mouse_pressed():
    global click_x, click_y, click_rotation_x, click_rotation_y
    click_x = mouse_x
    click_y = mouse_y
    click_rotation_x = rotation_x
    click_rotation_y = rotation_y


def mouse_entered():  # GAP: no mouse_entered() callback; mouse_in_window stays False and the attractor circle is never drawn
    global mouse_in_window
    mouse_in_window = True


def mouse_exited():  # GAP: no mouse_exited() callback
    global mouse_in_window
    mouse_in_window = False


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
