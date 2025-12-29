from processing import *

angle = 0.0

def setup():
    size(800, 600, 1.0)
    mode_3d()

def draw():
    camera_position(100.0, 100.0, 300.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(220)


    push_matrix()
    rotate(angle)
    geometry(box)
    pop_matrix()

    angle += 0.02


# TODO: this should happen implicitly on module load somehow
run()

