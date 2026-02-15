from processing import *

angle = 0.0

def setup():
    size(800, 600)
    mode_3d()

    # Directional Light
    dir_light = create_directional_light(0.5, 0.24, 1.0, 1500.0)

    # Point Lights
    point_light_a = create_point_light(1.0, 0.5, 0.25, 1000000.0, 200.0, 0.5)
    point_light_a.position(-25.0, 5.0, 51.0)
    point_light_a.look_at(0.0, 0.0, 0.0)

    point_light_b = create_point_light(0.0, 0.5, 0.75, 2000000.0, 200.0, 0.25)
    point_light_b.position(0.0, 5.0, 50.5)
    point_light_b.look_at(0.0, 0.0, 0.0)

    # Spot Light
    spot_light = create_spot_light(0.25, 0.8, 0.19, 15.0 * 1000000.0, 200.0, 0.84, 0.0, 0.7854)
    spot_light.position(40.0, 0.0, 70.0)
    spot_light.look_at(0.0, 0.0, 0.0)

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

