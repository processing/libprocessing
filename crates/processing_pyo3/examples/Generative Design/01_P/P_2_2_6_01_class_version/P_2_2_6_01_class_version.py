# P_2_2_6_01_class_version
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

# A chain of linked pendulums. Each a little shorter and faster than the one it's linked to.
# Each joint of the pendulum leaves behind its own trail.
#
# KEYS
# 1                   : toggle pendulum
# 2                   : toggle pendulum path
# -                   : decrease speed relation
# +                   : increase speed relation
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
    def __init__(self, x, y, joints, line_length, speed_relation, speed=None):
        self.center = Vec2(x, y)
        self.pendulum_path = []
        self.angle = 0
        self.max_angle = 360
        self.joints = joints
        self.line_length = line_length
        self.speed_relation = speed_relation
        if speed is None:
            speed = 8 / pow(1.75, self.joints - 1) / pow(2, speed_relation - 1)
        self.speed = speed

        # new empty array for each joint
        for i in range(self.joints):
            self.pendulum_path.append([])

    def draw(self):
        self.angle += self.speed

        if self.angle <= self.max_angle + self.speed:
            # start at the center position
            pos = self.center.copy()

            for i in range(self.joints):
                a = self.angle * pow(self.speed_relation, i)
                if i % 2 == 1:
                    a = -a

                next_pos = Vec2.from_angle(radians(a))

                next_pos.set_mag((self.line_length / self.joints) * (self.joints - i))
                next_pos.add(pos)

                if show_pendulum:
                    no_stroke()
                    fill(0.0, 0.1)
                    ellipse(pos.x, pos.y, 4, 4)
                    no_fill()
                    stroke(0.0, 0.1)
                    line(pos.x, pos.y, next_pos.x, next_pos.y)

                pos = next_pos

                self.pendulum_path[i].append(pos)

        if show_pendulum_path:
            stroke_weight(1.6)
            for i in range(len(self.pendulum_path)):
                path = self.pendulum_path[i]

                begin_shape()
                path_hue = remap(i, 0, self.joints, 120, 360)
                stroke(hsva(path_hue, 0.8, 0.6, 0.5))
                for j in range(len(path)):
                    vertex(path[j].x, path[j].y)
                end_shape()


shape = None

joints = 5
line_length = 100
speed_relation = 2

show_pendulum = True
show_pendulum_path = True


def setup():
    full_screen()
    no_fill()
    stroke_weight(1)
    start_drawing()


def draw():
    background(1.0)

    shape.draw()


def start_drawing():
    global shape
    shape = Shape(width / 2, height / 2, joints, line_length, speed_relation)


def key_pressed():
    global joints, line_length, speed_relation, show_pendulum, show_pendulum_path
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")

    if key_code == DELETE or key_code == BACKSPACE:
        start_drawing()

    if key_code == UP:
        line_length += 2
        start_drawing()
    if key_code == DOWN:
        line_length -= 2
        start_drawing()
    if key_code == LEFT_ARROW:
        joints -= 1
        if joints < 1:
            joints = 1
        start_drawing()
    if key_code == RIGHT_ARROW:
        joints += 1
        if joints > 10:
            joints = 10
        start_drawing()

    if key == "+":
        speed_relation += 0.5
        if speed_relation > 5:
            speed_relation = 5
        start_drawing()
    if key == "-":
        speed_relation -= 0.5
        if speed_relation < 2:
            speed_relation = 2
        start_drawing()

    if key == "1":
        show_pendulum = not show_pendulum
    if key == "2":
        show_pendulum_path = not show_pendulum_path


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
