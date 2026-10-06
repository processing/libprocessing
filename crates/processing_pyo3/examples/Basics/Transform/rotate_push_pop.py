# Rotate Push Pop.
#
# The push() and pop() functions allow for more control over transformations.
# The push function saves the current coordinate system to the stack
# and pop() restores the prior coordinate system.
from mewnala import *

a = 0.0  # Angle of rotation
offset = PI / 24.0  # Angle offset between boxes
num = 12  # Number of boxes


def setup():
    size(640, 360)
    mode_3d()
    no_stroke()
    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)


def draw():
    global a
    background(0.0, 0.0, 0.1)

    for i in range(num):
        gray = remap(i, 0, num - 1, 0, 1)
        push_matrix()
        fill(gray)
        rotate_y(a + offset * i)
        rotate_x(a / 2 + offset * i)
        box(200)
        pop_matrix()

    a += 0.01


run()
