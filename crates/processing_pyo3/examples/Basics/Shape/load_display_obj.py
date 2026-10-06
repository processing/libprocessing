# Load and Display an OBJ Shape.
#
# The load_model() command is used to read OBJ (Object) files into a
# sketch. This example loads an OBJ file of a rocket and displays it
# to the screen.
from mewnala import *

rocket = None
ry = 0.0


def setup():
    global rocket
    size(640, 360)
    mode_3d()
    rocket = load_model("data/rocket.obj")  # GAP: load_model() (OBJ with .mtl and texture) is missing
    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)


def draw():
    global ry
    background(0.0)

    translate(0, -100, -200)
    rotate_z(PI)
    rotate_y(ry)
    shape(rocket)  # GAP: shape(model) draws a loaded model

    ry += 0.02


run()
