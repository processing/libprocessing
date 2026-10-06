# Glossy Fish Eye
#
# A fish-eye shader is used on the main surface and
# a glossy specular reflection shader is used on the
# offscreen canvas.
from mewnala import *

fisheye = None
glossy = None
canvas = None
ball = None

use_fish_eye = True


def setup():
    global fisheye, glossy, canvas, ball
    size(640, 640)
    canvas = create_graphics(width, height)
    canvas.mode_3d()

    fisheye = load_shader("data/fish_eye.wesl")

    # the custom material does not see the scene lights or camera: the
    # light position and the (default) camera position are uniforms
    cam_z = (height / 2) / tan(PI / 6)
    glossy = create_material(load_shader("data/glossy.wesl"),
                             ambient_colour=(0.0, 0.0, 0.0),
                             diffuse_colour=(0.9, 0.2, 0.2),
                             specular_colour=(1.0, 1.0, 1.0),
                             ambient_intensity=1.0,
                             diffuse_intensity=1.0,
                             specular_intensity=0.7,
                             roughness=0.7,
                             sharpness=0.0,
                             light_position=(0.0, 0.0, 0.0),
                             camera_position=(0.0, 0.0, cam_z))

    ball = Geometry.sphere(50)


def draw():
    canvas.begin_draw()
    canvas.no_stroke()
    canvas.background(0.0)
    # pointLight(204, 204, 204, 1000, 1000, 1000) under rotateY(frameCount * 0.01),
    # converted from Processing's corner-origin, y-down space to the centered y-up world
    angle = frame_count * 0.01
    lx = 1000 * cos(angle) + 1000 * sin(angle)
    ly = 1000
    lz = -1000 * sin(angle) + 1000 * cos(angle)
    glossy.set(light_position=(lx - canvas.width / 2, canvas.height / 2 - ly, lz))
    canvas.material(glossy)
    x = 0
    while x < canvas.width + 100:
        y = 0
        while y < canvas.height + 100:
            z = 0
            while z < 400:
                canvas.push_matrix()
                canvas.translate(x - canvas.width / 2, canvas.height / 2 - y, -z)
                canvas.draw_geometry(ball)
                canvas.pop_matrix()
                z += 100
            y += 100
        x += 100
    canvas.end_draw()

    image(canvas, 0, 0, width, height)
    if use_fish_eye:
        filter(fisheye, aperture=180.0)


def mouse_pressed():
    global use_fish_eye
    use_fish_eye = not use_fish_eye


run()
