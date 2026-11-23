# M_1_5_03_TOOL
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

# noise values (noise 3d) are used to animate a bunch of agents.
#
# The controlP5 panel of the original is not ported; its sliders are the
# variables at the top of the sketch (the m key, which opened the menu, is gone).
#
# KEYS
# 1-2                 : switch noise mode
# space               : new noise seed
# backspace           : clear screen
# s                   : save png
from datetime import datetime

from processing_py import *

# ------ agents ------
agents = []  # create more ... to fit max slider agentsCount
agents_count = 4000  # slider 1 - 10000
noise_scale = 100  # slider 1 - 1000
noise_strength = 10  # slider 0 - 100
noise_z_range = 0.4  # slider 0 - 5, the panel called set_noise_z_range(noise_z_range) on every agent when it changed
overlay_alpha = 10  # slider 0 - 255
agents_alpha = 90  # slider 0 - 255
stroke_width = 0.3  # slider 0 - 10
draw_mode = 1


class Agent:
    def __init__(self):
        self.p = Vec2(random(width), random(height))
        self.p_old = Vec2(self.p.x, self.p.y)
        self.noise_z = 0
        self.noise_z_velocity = 0.01
        self.step_size = random(1, 5)
        self.angle = 0
        # init noiseZ
        self.set_noise_z_range(0.4)

    def update1(self):
        self.angle = noise(self.p.x / noise_scale, self.p.y / noise_scale, self.noise_z) * noise_strength

        self.p.x += cos(self.angle) * self.step_size
        self.p.y += sin(self.angle) * self.step_size

        # offscreen wrap
        if self.p.x < -10:
            self.p.x = self.p_old.x = width + 10
        if self.p.x > width + 10:
            self.p.x = self.p_old.x = -10
        if self.p.y < -10:
            self.p.y = self.p_old.y = height + 10
        if self.p.y > height + 10:
            self.p.y = self.p_old.y = -10

        stroke_weight(stroke_width * self.step_size)
        line(self.p_old.x, self.p_old.y, self.p.x, self.p.y)

        self.p_old.set(self.p)
        self.noise_z += self.noise_z_velocity

    def update2(self):
        self.angle = noise(self.p.x / noise_scale, self.p.y / noise_scale, self.noise_z) * 24
        self.angle = (self.angle - int(self.angle)) * noise_strength

        self.p.x += cos(self.angle) * self.step_size
        self.p.y += sin(self.angle) * self.step_size

        # offscreen wrap
        if self.p.x < -10:
            self.p.x = self.p_old.x = width + 10
        if self.p.x > width + 10:
            self.p.x = self.p_old.x = -10
        if self.p.y < -10:
            self.p.y = self.p_old.y = height + 10
        if self.p.y > height + 10:
            self.p.y = self.p_old.y = -10

        stroke_weight(stroke_width * self.step_size)
        line(self.p_old.x, self.p_old.y, self.p.x, self.p.y)

        self.p_old.set(self.p)
        self.noise_z += self.noise_z_velocity

    def set_noise_z_range(self, the_noise_z_range):
        # small values will increase grouping of the agents
        self.noise_z = random(the_noise_z_range)


def setup():
    size(1280, 800)

    for i in range(10000):
        agents.append(Agent())


def draw():
    fill(1.0, overlay_alpha / 255)
    no_stroke()
    rect(0, 0, width, height)

    stroke(0.0, agents_alpha / 255)
    # draw agents
    if draw_mode == 1:
        for i in range(agents_count):
            agents[i].update1()
    else:
        for i in range(agents_count):
            agents[i].update2()


def key_released():
    global draw_mode
    if key == "1":
        draw_mode = 1
    if key == "2":
        draw_mode = 2
    if key == "s" or key == "S":
        save_frame(timestamp() + ".png")
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
