# P_2_2_6_02
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

# Drawing tool that moves a pendulum-esq contraption along paths drawn by the mouse.
# Each joint of the pendulum leaves behind its own trail.
#
# MOUSE
# mouse               : click and drag to create a path to draw a pendulum along with
#
# KEYS
# 1                   : toggle path line
# 2                   : toggle pendulum
# 3                   : toggle pendulum path
# -                   : decrease speed
# +                   : increase speed
# arrow down          : decrease length of lines
# arrow up            : increase length of lines
# arrow left          : decrease joints
# arrow right         : increase joints
# del, backspace      : clear screen
# s                   : save png
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
from datetime import datetime

from processing_py import *


class Shape:
    def __init__(self, line_length, speed, resolution, joints):
        self.shape_path = []
        self.pendulum_path = []
        self.iterator = 0
        self.line_length = line_length
        self.speed = speed
        self.resolution = resolution
        self.joints = joints

        for i in range(self.joints):
            self.pendulum_path.append([])

    def add_pos(self, x, y):
        new_pos = Vec2(x, y)
        self.shape_path.append(new_pos)

    def draw(self):
        stroke_weight(0.8)
        stroke(0.0, 0.1)

        if show_path:
            begin_shape()
            for pos in self.shape_path:
                vertex(pos.x, pos.y)
            end_shape()

        current_index = floor(self.iterator)

        if current_index > 0:
            previous_pos = self.shape_path[current_index - 1]
            if current_index < len(self.shape_path):
                current_pos = self.shape_path[current_index]
                offset_pos_a = previous_pos.lerp(current_pos, self.iterator - current_index)
            else:
                # at the end of the path the pendulum stays on the last position
                offset_pos_a = previous_pos.copy()
            for i in range(self.joints):
                offset_pos_b = Vec2.from_angle(
                    (PI / (i + 1)) +
                    self.iterator /
                    pow(-2, self.joints - i) * self.speed
                )
                offset_pos_b.set_mag((self.line_length / self.joints) * (self.joints - i))
                offset_pos_b.add(offset_pos_a)

                if show_pendulum:
                    fill(0.0, 0.1)
                    ellipse(offset_pos_a.x, offset_pos_a.y, 4, 4)
                    no_fill()
                    line(offset_pos_a.x, offset_pos_a.y, offset_pos_b.x, offset_pos_b.y)

                offset_pos_a = offset_pos_b

                self.pendulum_path[i].append(offset_pos_a)

            if show_pendulum_path:
                stroke_weight(1.6)
                for index in range(len(self.pendulum_path)):
                    path = self.pendulum_path[index]
                    begin_shape()
                    stroke(hsva(360 / self.joints * index, 0.8, 0.6, 0.5))
                    for pos in path:
                        curve_vertex(pos.x, pos.y)
                    end_shape()

    def update(self):
        self.iterator += self.resolution
        self.iterator = constrain(self.iterator, 0, len(self.shape_path))


shapes = []

new_shape = None

joints = 8
line_length = 32
speed = 16
resolution = 0.2

show_path = True
show_pendulum = True
show_pendulum_path = True


def setup():
    full_screen()
    no_fill()
    stroke_weight(1)


def draw():
    background(1.0)

    for shape in shapes:
        shape.draw()
        shape.update()

    if new_shape is not None:
        new_shape.add_pos(mouse_x, mouse_y)
        new_shape.draw()
        new_shape.update()


def mouse_pressed():
    global new_shape
    new_shape = Shape(line_length, speed, resolution, joints)
    new_shape.add_pos(mouse_x, mouse_y)


def mouse_released():
    global new_shape
    shapes.append(new_shape)
    new_shape = None


def key_pressed():
    global shapes, new_shape, joints, line_length, speed
    global show_path, show_pendulum, show_pendulum_path
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key_code == DELETE or key_code == BACKSPACE:
        shapes = []
        new_shape = None

    if key_code == UP:
        line_length += 2
    if key_code == DOWN:
        line_length -= 2
    if key_code == LEFT_ARROW:
        joints -= 1
        joints = max(1, joints)
    if key_code == RIGHT_ARROW:
        joints += 1
        joints = max(1, joints)

    if key == "1":
        show_path = not show_path
    if key == "2":
        show_pendulum = not show_pendulum
    if key == "3":
        show_pendulum_path = not show_pendulum_path

    if key == "+":
        speed += 0.5
    if key == "-":
        speed -= 0.5


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
