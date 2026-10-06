# On/Off.
#
# Uses the default lights to show a simple box. The lights() function
# is used to turn on the default lighting. Click the mouse to turn the
# lights off.
from mewnala import *

spin = 0.0
light = None


def setup():
    global light
    size(640, 360)
    mode_3d()
    no_stroke()
    light = directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))


def draw():
    global spin
    background(0.2)

    # A lit material turns the lights on; unlit() turns them off
    if not mouse_is_pressed:
        roughness(0.6)
    else:
        unlit()

    spin += 0.01

    push_matrix()
    # Rotations about x run the other way in the y-up world
    rotate_x(-PI / 9)
    rotate_y(PI / 5 + spin)
    box(150)
    pop_matrix()


run()
