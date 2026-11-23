# P_2_2_6_01
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

shape = None

joints = 5
line_length = 100
speed_relation = 2
center = None
pendulum_path = []
angle = 0
max_angle = 360
speed = 0

show_pendulum = True
show_pendulum_path = True


def setup():
    global center
    full_screen()
    no_fill()
    stroke_weight(1)

    center = Vec2(width / 2, height / 2)

    start_drawing()


def start_drawing():
    global pendulum_path, angle, speed
    pendulum_path = []
    # new empty array for each joint
    for i in range(joints):
        pendulum_path.append([])

    angle = 0
    speed = 8 / pow(1.75, joints - 1) / pow(2, speed_relation - 1)


def draw():
    global angle
    background(1.0)

    angle += speed

    # each frame, create new positions for each joint
    if angle <= max_angle + speed:
        # start at the center position
        pos = center.copy()

        for i in range(joints):
            a = angle * pow(speed_relation, i)
            if i % 2 == 1:
                a = -a
            next_pos = Vec2.from_angle(radians(a))
            next_pos.set_mag((joints - i) / joints * line_length)
            next_pos.add(pos)

            if show_pendulum:
                no_stroke()
                fill(0.0, 0.1)
                ellipse(pos.x, pos.y, 4, 4)
                no_fill()
                stroke(0.0, 0.1)
                line(pos.x, pos.y, next_pos.x, next_pos.y)

            pendulum_path[i].append(next_pos)
            pos = next_pos

    # draw the path for each joint
    if show_pendulum_path:
        stroke_weight(1.6)
        for i in range(len(pendulum_path)):
            path = pendulum_path[i]

            begin_shape()
            path_hue = remap(i, 0, joints, 120, 360)
            stroke(hsva(path_hue, 0.8, 0.6, 0.5))
            for j in range(len(path)):
                vertex(path[j].x, path[j].y)
            end_shape()


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
