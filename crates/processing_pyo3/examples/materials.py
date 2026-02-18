from processing import *

mat = None

def setup():
    global mat
    size(800, 600)
    mode_3d()

    dir_light = create_directional_light(1.0, 0.98, 0.95, 1500.0)
    point_light = create_point_light(1.0, 1.0, 1.0, 100000.0, 800.0, 0.0)
    point_light.position(200.0, 200.0, 400.0)

    mat = Material()
    mat.set_float("roughness", 0.3)
    mat.set_float("metallic", 0.8)
    mat.set_float4("base_color", 1.0, 0.85, 0.57, 1.0)

def draw():
    camera_position(0.0, 0.0, 200.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(12, 12, 18)

    use_material(mat)
    draw_sphere(50.0)

run()
