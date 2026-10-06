# Perspective.
#
# Move the mouse left and right to change the field of view (fov).
# Click to modify the aspect ratio. The perspective() function
# sets a perspective projection applying foreshortening, making
# distant objects appear smaller than closer ones. The parameters
# define a viewing volume with the shape of truncated pyramid.
# Objects near to the front of the volume appear their actual size,
# while farther objects appear smaller. This projection simulates
# the perspective of the world more accurately than orthographic projection.
# The version of perspective without parameters sets the default
# perspective and the version with four parameters allows the programmer
# to set the area precisely.
from mewnala import *

light = None


def setup():
    global light
    size(640, 360)
    mode_3d()
    no_stroke()
    light = directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)


def draw():
    background(0.0)
    camera_y = height / 2.0
    # A fov of 0 would divide by zero below
    fov = max(mouse_x, 1) / width * PI / 2
    camera_z = camera_y / tan(fov / 2.0)
    aspect = width / height
    if mouse_is_pressed:
        aspect = aspect / 2.0
    perspective(fov, aspect, camera_z / 10.0, camera_z * 10.0)

    translate(30, 0, 0)
    # Rotations about x run the other way in the y-up world
    rotate_x(PI / 6)
    rotate_y(PI / 3 + mouse_y / height * PI)
    box(45)
    translate(0, 0, -50)
    box(30)


run()
