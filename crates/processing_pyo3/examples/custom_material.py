from mewnala import *

mat = None

def setup():
    global mat
    size(800, 600)
    mode_3d()

    shader = Shader.load("shaders/custom_material.wesl")
    mat = Material(shader, color=[1.0, 0.2, 0.4, 1.0])

def draw():
    camera_position(0.0, 0.0, 200.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(12, 12, 18)

    use_material(mat)
    box(80.0, 80.0, 80.0)

run()
