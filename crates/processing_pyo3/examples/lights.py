from processing import *

angle = 0.0

def setup():
    size(800, 600)
    mode_3d()
    dir_light = create_directional_light(0.5, 0.24, 1.0, 1500.0)

def draw():
    global angle
    camera_position(100.0, 100.0, 300.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(220)

    push_matrix()
    rotate(angle)
    draw_box(100.0, 100.0, 100.0)
    pop_matrix()

    angle += 0.02


# TODO: this should happen implicitly on module load somehow
run()

