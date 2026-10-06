# Toon Shading.
#
# Example showing the use of a custom lighting shader in order
# to apply a "toon" effect on the scene. Based on the glsl tutorial
# from lighthouse 3D:
# http://www.lighthouse3d.com/tutorials/glsl-tutorial/toon-shader-version-ii/
from mewnala import *

toon = None
plain = None
light = None
shader_enabled = True


def setup():
    global toon, plain, light
    size(640, 360)
    mode_3d()
    no_stroke()
    fill(0.8)
    toon = create_material(load_shader("data/toon.wesl"), light_dir=(0.0, 0.0, 1.0))
    # the default (lit) material used when the toon shader is off
    plain = create_material(albedo=(0.8, 0.8, 0.8), roughness=0.6)
    light = directional_light((0.8, 0.8, 0.8), 4000.0)


def draw():
    no_stroke()
    background(0.0)
    dir_y = (mouse_y / height - 0.5) * 2
    dir_x = (mouse_x / width - 0.5) * 2
    # directionalLight(204, 204, 204, -dirX, -dirY, -1), y flipped for the y-up world
    light.look_at(-dir_x, dir_y, -1)
    if shader_enabled:
        # the custom material does not see the scene lights: pass the
        # direction toward the light as a uniform
        toon.set(light_dir=(dir_x, -dir_y, 1.0))
        material(toon)
    else:
        material(plain)  # GAP: no reset_shader()/no_material() to return to the default material
    sphere(120)


def mouse_pressed():
    global shader_enabled
    shader_enabled = not shader_enabled


run()
