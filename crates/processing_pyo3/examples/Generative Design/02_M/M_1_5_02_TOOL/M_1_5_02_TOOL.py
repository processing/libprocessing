# M_1_5_02_TOOL
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

# noise values (noise 2d) are used to animate a bunch of agents.
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
noise_scale = 300  # slider 1 - 1000
noise_strength = 10  # slider 0 - 100
overlay_alpha = 10  # slider 0 - 255
agents_alpha = 90  # slider 0 - 255
stroke_width = 0.3  # slider 0 - 10
draw_mode = 1


class Agent:
    def __init__(self):
        self.p = Vec2(random(width), random(height))
        self.p_old = Vec2(self.p.x, self.p.y)
        self.step_size = random(1, 5)
        self.angle = 0
        self.is_outside = False

    def update1(self):
        self.angle = noise(self.p.x / noise_scale, self.p.y / noise_scale) * noise_strength

        self.p.x += cos(self.angle) * self.step_size
        self.p.y += sin(self.angle) * self.step_size

        if self.p.x < -10:
            self.is_outside = True
        elif self.p.x > width + 10:
            self.is_outside = True
        elif self.p.y < -10:
            self.is_outside = True
        elif self.p.y > height + 10:
            self.is_outside = True

        if self.is_outside:
            self.p.x = random(width)
            self.p.y = random(height)
            self.p_old.set(self.p)

        stroke_weight(stroke_width * self.step_size)
        line(self.p_old.x, self.p_old.y, self.p.x, self.p.y)

        self.p_old.set(self.p)

        self.is_outside = False

    def update2(self):
        self.angle = noise(self.p.x / noise_scale, self.p.y / noise_scale) * 24
        self.angle = (self.angle - int(self.angle)) * noise_strength

        self.p.x += cos(self.angle) * self.step_size
        self.p.y += sin(self.angle) * self.step_size

        if self.p.x < -10:
            self.is_outside = True
        elif self.p.x > width + 10:
            self.is_outside = True
        elif self.p.y < -10:
            self.is_outside = True
        elif self.p.y > height + 10:
            self.is_outside = True

        if self.is_outside:
            self.p.x = random(width)
            self.p.y = random(height)
            self.p_old.set(self.p)

        stroke_weight(stroke_width * self.step_size)
        line(self.p_old.x, self.p_old.y, self.p.x, self.p.y)

        self.p_old.set(self.p)

        self.is_outside = False


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
    if key == " ":
        new_noise_seed = int(random(100000))
        print("newNoiseSeed: " + str(new_noise_seed))
        noise_seed(new_noise_seed)
    if key_code == DELETE or key_code == BACKSPACE:
        background(1.0)


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
