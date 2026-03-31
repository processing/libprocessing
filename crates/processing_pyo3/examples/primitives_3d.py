from mewnala import *

t = 0.0

def setup():
    size(900, 400)
    mode_3d()

def draw():
    global t
    camera_position(0, 80, 800)
    camera_look_at(0, 0, 0)
    background(15, 15, 20)

    fill(255)
    roughness(0.35)

    offsets = [-315, -225, -135, -45, 45, 135, 225, 315]

    for i, x in enumerate(offsets):
        push_matrix()
        translate(x, 0)
        rotate(t)

        if i == 0:
            box(50, 50, 50)
        elif i == 1:
            sphere(30, 24, 16)
        elif i == 2:
            cylinder(25, 60, 24)
        elif i == 3:
            cone(25, 60, 24)
        elif i == 4:
            torus(30, 10, 24, 16)
        elif i == 5:
            capsule(15, 40, 24)
        elif i == 6:
            conical_frustum(15, 25, 60, 24)
        elif i == 7:
            tetrahedron(30)

        pop_matrix()

    t += 0.015

run()
