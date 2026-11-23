# M_1_4_01
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

# creates a terrain like mesh based on noise values.
#
# MOUSE
# position x/y + left drag   : specify noise input range
# position x/y + right drag  : camera controls
#
# KEYS
# l                          : toogle display strokes on/off
# arrow up                   : noise falloff +
# arrow down                 : noise falloff -
# arrow left                 : noise octaves -
# arrow right                : noise octaves +
# space                      : new noise seed
# +                          : zoom in
# -                          : zoom out
# s                          : save png
from datetime import datetime

from processing_py import *

# ------ mesh ------
tile_count = 50
z_scale = 150

# ------ noise ------
noise_x_range = 10
noise_y_range = 10
octaves = 4
falloff = 0.5

# ------ mesh coloring ------
mid_color = None
top_color = None
bottom_color = None
stroke_color = None
threshold = 0.30

# ------ mouse interaction ------
offset_x = 0
offset_y = 0
click_x = 0
click_y = 0
zoom = -280
rotation_x = 0
rotation_z = 0
target_rotation_x = -PI / 3
target_rotation_z = 0
click_rotation_x = 0
click_rotation_z = 0

# ------ image output ------
show_stroke = True


def setup():
    global top_color, mid_color, bottom_color, stroke_color
    size(800, 800)
    mode_3d()
    roughness(0.6)
    cursor(CROSS)

    # colors
    top_color = hsva(0, 0.0, 1.0)
    mid_color = hsva(191, 0.99, 0.63)
    bottom_color = hsva(0, 0.0, 0.0)

    stroke_color = hsva(0, 0.0, 0.0)


def draw():
    global offset_x, offset_y, target_rotation_x, target_rotation_z
    global rotation_x, rotation_z, noise_x_range, noise_y_range

    if show_stroke:
        stroke(stroke_color)
    else:
        no_stroke()

    background(1.0)
    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))

    # ------ set view ------
    push_matrix()
    translate(0, 0, zoom)

    if mouse_is_pressed and mouse_button == RIGHT:
        offset_x = mouse_x - click_x
        offset_y = mouse_y - click_y
        target_rotation_x = min(max(click_rotation_x + offset_y / width * TWO_PI, -HALF_PI), HALF_PI)
        target_rotation_z = click_rotation_z + offset_x / height * TWO_PI
    rotation_x += (target_rotation_x - rotation_x) * 0.25
    rotation_z += (target_rotation_z - rotation_z) * 0.25
    rotate_x(-rotation_x)
    rotate_z(-rotation_z)

    # ------ mesh noise ------
    if mouse_is_pressed and mouse_button == LEFT:
        noise_x_range = int(mouse_x / 10)
        noise_y_range = int(mouse_y / 10)

    noise_detail(octaves, falloff)
    noise_y_max = 0

    tile_size_y = height / tile_count
    noise_step_y = noise_y_range / tile_count

    for mesh_y in range(tile_count + 1):
        # each row is one triangle strip, rebuilt every frame
        strip = create_geometry(topology=TRIANGLE_STRIP)
        for mesh_x in range(tile_count + 1):

            x = remap(mesh_x, 0, tile_count, -width / 2, width / 2)
            y = remap(mesh_y, 0, tile_count, -height / 2, height / 2)

            noise_x = remap(mesh_x, 0, tile_count, 0, noise_x_range)
            noise_y = remap(mesh_y, 0, tile_count, 0, noise_y_range)
            z1 = noise(noise_x, noise_y)
            z2 = noise(noise_x, noise_y + noise_step_y)

            noise_y_max = max(noise_y_max, z1)
            if z1 <= threshold:
                amount = remap(z1, 0, threshold, 0.15, 1)
                inter_color = lerp_color(bottom_color, mid_color, amount)
            else:
                amount = remap(z1, threshold, noise_y_max, 0, 1)
                inter_color = lerp_color(mid_color, top_color, amount)
            strip.color(inter_color)

            strip.vertex(x, y, z1 * z_scale)
            strip.vertex(x, y + tile_size_y, z2 * z_scale)
        draw_geometry(strip)
    pop_matrix()


def mouse_pressed():
    global click_x, click_y, click_rotation_x, click_rotation_z
    click_x = mouse_x
    click_y = mouse_y
    click_rotation_x = rotation_x
    click_rotation_z = rotation_z


def key_pressed():
    global falloff, octaves, zoom
    if key_code == UP:
        falloff += 0.05
    if key_code == DOWN:
        falloff -= 0.05
    if falloff > 1.0:
        falloff = 1.0
    if falloff < 0.0:
        falloff = 0.0

    if key_code == LEFT_ARROW:
        octaves -= 1
    if key_code == RIGHT_ARROW:
        octaves += 1
    if octaves < 0:
        octaves = 0

    if key == "+":
        zoom += 20
    if key == "-":
        zoom -= 20


def key_released():
    global show_stroke
    if key == "s" or key == "S":
        save_frame(timestamp() + "_##.png")
    if key == "l" or key == "L":
        show_stroke = not show_stroke
    if key == " ":
        noise_seed(int(random(100000)))


def timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


run()
