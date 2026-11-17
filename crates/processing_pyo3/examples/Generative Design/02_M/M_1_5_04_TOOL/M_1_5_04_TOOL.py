# M_1_5_04_TOOL
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
# 1                   : draw style line
# 2                   : draw style ellipse
# backspace           : clear screen
# s                   : save png
from datetime import datetime

from processing_py import *

# ------ agents ------
agents = []  # create more ... to fit max slider agentsCount
agents_count = 2000  # slider 1 - 10000
noise_scale = 100  # slider 1 - 1000
noise_strength = 10  # slider 0 - 100
noise_sticking_range = 0.4  # slider 0 - 5, the panel called set_noise_sticking(noise_sticking_range) on every agent when it changed
overlay_alpha = 8  # slider 0 - 255
agents_alpha = 90  # slider 0 - 255
stroke_width = 2  # slider 0 - 10
agent_width_min = 1.5  # range slider 0 - 50 (lower end)
agent_width_max = 15  # range slider 0 - 50 (upper end)
draw_mode = 1


class Agent:
    def __init__(self):
        self.p = Vec2(random(width), random(height))
        self.p_old = Vec2(self.p.x, self.p.y)
        self.z_noise = 0
        self.z_noise_velocity = 0.01
        self.angle = 0
        self.randomizer = random(1)
        self.step_size = 1 + self.randomizer * 4
        # colors
        if self.randomizer < 0.5:
            self.col = hsva(int(random(170, 190)), 0.7, int(random(0, 100)) / 100)
        else:
            self.col = hsva(int(random(40, 60)), 0.7, int(random(0, 100)) / 100)

        # init zNoise
        self.set_noise_sticking(0.4)

    def update1(self):
        self.angle = noise(self.p.x / noise_scale, self.p.y / noise_scale, self.z_noise) * noise_strength

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

        stroke(self.col.with_alpha(agents_alpha / 255))
        stroke_weight(stroke_width)
        line(self.p_old.x, self.p_old.y, self.p.x, self.p.y)

        agent_width = remap(self.randomizer, 0, 1, agent_width_min, agent_width_max)
        push_matrix()
        translate(self.p_old.x, self.p_old.y)
        rotate(atan2(self.p.y - self.p_old.y, self.p.x - self.p_old.x))
        line(0, -agent_width, 0, agent_width)
        pop_matrix()

        self.p_old.set(self.p)
        self.z_noise += self.z_noise_velocity

    def update2(self):
        self.angle = noise(self.p.x / noise_scale, self.p.y / noise_scale, self.z_noise) * noise_strength

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

        stroke(self.col)
        stroke_weight(2)
        agent_width = remap(self.randomizer, 0, 1, agent_width_min, agent_width_max) * 2
        ellipse(self.p_old.x, self.p_old.y, agent_width, agent_width)

        self.p_old.set(self.p)
        self.z_noise += self.z_noise_velocity

    def set_noise_sticking(self, sticking_range):
        # small values will increase stinking of the agents
        self.z_noise = random(sticking_range)


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
    for i in range(agents_count):
        if draw_mode == 1:
            agents[i].update1()
        else:
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
