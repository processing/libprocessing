# P_2_2_6_04
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

# Drawing tool that moves a branching pendulum contraption along paths drawn by the mouse.
# The last joint of the pendulum leaves behind its own trail.
#
# MOUSE
# mouse               : click and drag to create a path to draw a pendulum along with
#
# KEYS
# 1                   : toggle path line
# 2                   : toggle pendulum
# 3                   : toggle pendulum path
# 4                   : toggle fill mode
# arrow up            : increase length of lines
# arrow down          : decrease length of lines
# arrow left          : decrease gravity
# arrow right         : increase gravity
# del, backspace      : clear screen
# s                   : save png
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
from datetime import datetime

from processing_py import *


class Shape:
    def __init__(self, pendulum_path_color):
        self.shape_path = []
        self.pendulum_path = []
        self.pendulum_path_color = pendulum_path_color
        self.iterator = 0
        self.line_length = line_length
        self.resolution = resolution
        self.pendulum = Pendulum(self.line_length, joints)

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

        if show_pendulum_path and len(self.pendulum_path) > 0:
            stroke_weight(1)
            stroke(self.pendulum_path_color)

            if fill_mode:
                c = self.pendulum_path_color
                fill(hsva(hue(c), saturation(c), brightness(c), 0.1))
                for group in self.pendulum_path:
                    begin_shape()
                    for pos in group:
                        vertex(pos.x, pos.y)
                    end_shape()
                no_fill()
            else:
                for column in range(len(self.pendulum_path[0])):
                    begin_shape()
                    for pos in self.pendulum_path:
                        vertex(pos[column].x, pos[column].y)
                    end_shape()

        if self.iterator < len(self.shape_path):
            current_index = floor(self.iterator)

            current_pos = self.shape_path[current_index]
            if current_index > 0:
                previous_pos = self.shape_path[current_index - 1]
                offset_pos = previous_pos.lerp(current_pos, self.iterator - current_index)
                heading = atan2(current_pos.y - previous_pos.y, current_pos.x - previous_pos.x) - HALF_PI

                push()
                translate(offset_pos.x, offset_pos.y)
                self.pendulum.update(heading)
                if show_pendulum:
                    self.pendulum.draw()
                pop()

                self.pendulum_path.append(self.pendulum.get_trail(offset_pos))

    def update(self):
        self.iterator += self.resolution
        self.iterator = constrain(self.iterator, 0, len(self.shape_path))


class Pendulum:
    def __init__(self, size, hierarchy):
        self.hierarchy = hierarchy - 1
        self.arm_count = floor(random(1, max_arms + 1))
        self.pendulum_arms = []
        self.size = size
        self.angle = random(TAU)
        self.origin = Vec2(0, 0)
        self.end = Vec2(0, 0)
        self.gravity = gravity
        self.damping = damping
        self.angular_acceleration = 0
        self.angular_velocity = 0

        i = 0
        while i < self.arm_count and self.hierarchy > 0:
            arm_size = self.size / (1.5 + random_gaussian() * arm_size_deviation)
            self.pendulum_arms.append(Pendulum(arm_size, self.hierarchy))
            i += 1

    def update(self, heading):
        self.end.set(self.origin.x + self.size * sin(self.angle), self.origin.y + self.size * cos(self.angle))

        self.angular_acceleration = (-self.gravity / self.size) * sin(self.angle + heading)
        self.angle += self.angular_velocity
        self.angular_velocity += self.angular_acceleration
        self.angular_velocity *= self.damping

        for pendulum_arm in self.pendulum_arms:
            pendulum_arm.update(heading)

    def get_trail(self, offset, pendulum_trail_paths=None):
        if pendulum_trail_paths is None:
            pendulum_trail_paths = []

        offset = offset.copy()
        offset.add(self.end)

        for pendulum_arm in self.pendulum_arms:
            if len(pendulum_arm.pendulum_arms) > 0:
                pendulum_arm.get_trail(offset, pendulum_trail_paths)
            else:
                trail = offset.copy()
                trail.add(pendulum_arm.end)
                pendulum_trail_paths.append(trail)

        return pendulum_trail_paths

    def draw(self):
        stroke(0.0, 0.4)
        begin_shape()
        vertex(self.origin.x, self.origin.y)
        vertex(self.end.x, self.end.y)
        end_shape()

        fill(0.0, 0.2)
        ellipse(self.end.x, self.end.y, 2, 2)
        no_fill()

        for pendulum_arm in self.pendulum_arms:
            push()
            translate(self.end.x, self.end.y)
            pendulum_arm.draw()
            pop()


shapes = []

new_shape = None

joints = 5
line_length = 32
resolution = 0.04
gravity = 0.099
damping = 0.995
max_arms = 3
arm_size_deviation = 0.2

show_path = True
show_pendulum = True
show_pendulum_path = True
fill_mode = True


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
    new_shape = Shape(hsva(random(360), 0.8, 0.6, 0.5))
    new_shape.add_pos(mouse_x, mouse_y)


def mouse_released():
    global new_shape
    shapes.append(new_shape)
    new_shape = None


def key_pressed():
    global shapes, new_shape, line_length, gravity
    global show_path, show_pendulum, show_pendulum_path, fill_mode
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
        gravity -= 0.001
    if key_code == RIGHT_ARROW:
        gravity += 0.001

    if key == "1":
        show_path = not show_path
    if key == "2":
        show_pendulum = not show_pendulum
    if key == "3":
        show_pendulum_path = not show_pendulum_path
    if key == "4":
        fill_mode = not fill_mode


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
