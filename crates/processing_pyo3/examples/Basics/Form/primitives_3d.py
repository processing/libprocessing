# Primitives 3D.
#
# Placing mathematically 3D objects in synthetic space.
# The lights() method reveals their imagined dimension.
# The box() and sphere() functions each have one parameter
# which is used to specify their size. These shapes are
# positioned using the translate() function.
from mewnala import *


def setup():
    size(640, 360)
    mode_3d()
    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)
    no_loop()


def draw():
    background(0.0)

    no_stroke()
    push_matrix()
    translate(130 - width / 2, 0, 0)
    rotate_y(1.25)
    rotate_x(-0.4)
    box(100)
    pop_matrix()

    no_fill()
    stroke(1.0)
    push_matrix()
    translate(500 - width / 2, height / 2 - height * 0.35, -200)
    sphere(280)
    pop_matrix()


run()
