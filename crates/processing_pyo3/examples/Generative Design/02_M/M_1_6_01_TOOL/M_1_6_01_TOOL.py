# M_1_6_01_TOOL
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

# noise values (noise 3d) are used to animate the movement of agents.
# each agent is visualizing its position history with a ribbon (line style).
#
# The controlP5 panel of the original is not ported; its sliders are the
# variables at the top of the sketch (the m key, which opened the menu, is gone).
# The high resolution export (TileSaver, p key) is not ported either.
#
# MOUSE
# position x/y + right drag  : camera controls
#
# KEYS
# f                          : toogle freeze noise animation
# space                      : new noise seed
# arrow up                   : zoom in
# arrow down                 : zoom out
# s                          : save png
from datetime import datetime

from processing_py import *

# ------ agents ------
agents_count = 600  # slider 1 - 1000
agents = []
noise_scale = 150  # slider 1 - 1000
noise_strength = 20  # slider 0 - 100
agent_alpha = 76  # slider 0 - 255
agent_color = None

# ------ mouse interaction ------
offset_x = 0
offset_y = 0
click_x = 0
click_y = 0
zoom = -100
rotation_x = 0
rotation_y = 0
target_rotation_x = 0
target_rotation_y = 0
click_rotation_x = 0
click_rotation_y = 0
space_size_x = 200
space_size_y = 300
space_size_z = 200
freeze = False


class Agent:
    def __init__(self):
        self.is_outside = False
        self.p = Vec3(0, 0, 0)
        self.set_random_postition()
        self.offset = 10000
        self.step_size = random(1, 8)
        self.angle_y = 0
        self.angle_z = 0
        # how many points has the ribbon
        self.ribbon = Ribbon3d(self.p, int(random(50, 300)))

    def update(self):
        self.angle_y = noise(self.p.x / noise_scale, self.p.y / noise_scale, self.p.z / noise_scale) * noise_strength
        self.angle_z = noise(self.p.x / noise_scale + self.offset, self.p.y / noise_scale, self.p.z / noise_scale) * noise_strength

        # convert polar to cartesian coordinates
        # stepSize is distance of the point to the last point
        # angleY is the angle for rotation around y-axis
        # angleZ is the angle for rotation around z-axis
        self.p.x += cos(self.angle_z) * cos(self.angle_y) * self.step_size
        self.p.y += sin(self.angle_z) * self.step_size
        self.p.z += cos(self.angle_z) * sin(self.angle_y) * self.step_size

        # boundingbox wrap
        if (self.p.x < -space_size_x or self.p.x > space_size_x or
                self.p.y < -space_size_y or self.p.y > space_size_y or
                self.p.z < -space_size_z or self.p.z > space_size_z):
            self.set_random_postition()
            self.is_outside = True

        # create ribbons
        self.ribbon.update(self.p, self.is_outside)
        self.is_outside = False

    def draw(self):
        self.ribbon.draw_line_ribbon(agent_color, 2.0)
        # self.ribbon.draw_mesh_ribbon(agent_color, 10.0)

    def set_random_postition(self):
        self.p.x = random(-space_size_x, space_size_x)
        self.p.y = random(-space_size_y, space_size_y)
        self.p.z = random(-space_size_z, space_size_z)


class Ribbon3d:
    def __init__(self, the_p, the_count):
        self.count = the_count  # how many points has the ribbon
        self.p = []
        self.is_gap = []
        for i in range(self.count):
            self.p.append(Vec3(the_p.x, the_p.y, the_p.z))
            self.is_gap.append(False)

    def update(self, the_p, the_is_gap):
        # shift the values to the right side
        # simple queue
        for i in range(self.count - 1, 0, -1):
            self.p[i].set(self.p[i - 1])
            self.is_gap[i] = self.is_gap[i - 1]
        self.p[0].set(the_p)
        self.is_gap[0] = the_is_gap

    def draw_mesh_ribbon(self, the_mesh_col, the_width):
        # draw the ribbons with meshes
        fill(the_mesh_col)
        no_stroke()

        # a quad strip has the same vertex order as a triangle strip
        strip = create_geometry(topology=TRIANGLE_STRIP)
        for i in range(self.count - 1):
            # if the point was wraped -> finish the mesh an start a new one
            if self.is_gap[i] == True:
                strip.vertex(self.p[i].x, self.p[i].y, self.p[i].z)
                strip.vertex(self.p[i].x, self.p[i].y, self.p[i].z)
                draw_geometry(strip)
                strip = create_geometry(topology=TRIANGLE_STRIP)
            else:
                v1 = self.p[i] - self.p[i + 1]
                v2 = self.p[i + 1] + self.p[i]
                v3 = v1.cross(v2)
                v2 = v1.cross(v3)
                # v1 = v1.normalize()
                v2 = v2.normalize()
                # v3 = v3.normalize()
                # v1.mult(the_width)
                v2.mult(the_width)
                # v3.mult(the_width)
                strip.vertex(self.p[i].x + v2.x, self.p[i].y + v2.y, self.p[i].z + v2.z)
                strip.vertex(self.p[i].x - v2.x, self.p[i].y - v2.y, self.p[i].z - v2.z)
        draw_geometry(strip)  # GAP: drawing a Geometry that got no vertices (a ribbon ending on a gap) logs a bevy slab_allocator "Use-after-free" error

    def draw_line_ribbon(self, the_stroke_col, the_width):
        # draw the ribbons with lines
        no_fill()
        stroke_weight(the_width)  # GAP: stroke_weight() is ignored by a LINES Geometry, the lines stay hairlines
        stroke(the_stroke_col)

        lines = create_geometry(topology=LINES)
        for i in range(self.count - 1):
            # if the point was wraped -> finish the line an start a new one
            if not self.is_gap[i] == True:
                lines.vertex(self.p[i].x, self.p[i].y, self.p[i].z)
                lines.vertex(self.p[i + 1].x, self.p[i + 1].y, self.p[i + 1].z)
        draw_geometry(lines)


def setup():
    size(800, 800)
    mode_3d()
    roughness(0.6)

    for i in range(1000):
        agents.append(Agent())


def draw():
    global offset_x, offset_y, target_rotation_x, target_rotation_y
    global rotation_x, rotation_y, agent_color

    background(1.0)
    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))

    push_matrix()

    # ------ set view ------
    translate(0, 0, zoom)
    print(str(mouse_is_pressed) + ", " + str(mouse_button) + ", " + str(RIGHT))
    if mouse_is_pressed and mouse_button == RIGHT:
        offset_x = mouse_x - click_x
        offset_y = mouse_y - click_y
        target_rotation_x = min(max(click_rotation_x + offset_y / width * TWO_PI, -HALF_PI), HALF_PI)
        target_rotation_y = click_rotation_y + offset_x / height * TWO_PI
    rotation_x += (target_rotation_x - rotation_x) * 0.25
    rotation_y += (target_rotation_y - rotation_y) * 0.25
    rotate_x(-rotation_x)
    rotate_y(rotation_y)

    no_fill()
    stroke(hsva(192, 1.0, 0.64))
    stroke_weight(1)
    box(space_size_x * 2, space_size_y * 2, space_size_z * 2)

    # ------ update and draw agents ------
    agent_color = color(0.0, agent_alpha / 255)
    for i in range(agents_count):
        if freeze == False:
            agents[i].update()
        agents[i].draw()
    pop_matrix()


# ------ interactions ------
def key_pressed():
    global zoom
    if key_code == UP:
        zoom += 20
    if key_code == DOWN:
        zoom -= 20


def key_released():
    global freeze
    if key == "s" or key == "S":
        save_frame(timestamp() + ".png")
    if key == "f" or key == "F":
        freeze = not freeze
    if key == " ":
        new_noise_seed = int(random(100000))
        print("newNoiseSeed: " + str(new_noise_seed))
        noise_seed(new_noise_seed)


def mouse_pressed():
    global click_x, click_y, click_rotation_x, click_rotation_y
    click_x = mouse_x
    click_y = mouse_y
    click_rotation_x = rotation_x
    click_rotation_y = rotation_y


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
