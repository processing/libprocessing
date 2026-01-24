from processing import *
from math import sin, cos
from random import gauss

geometry = None

def setup():
    global geometry
    size(800, 600)
    mode_3d()
    create_geometry()

def draw():
    global geometry

    camera_position(150.0, 150.0, 150.0)
    camera_look_at( 0.0, 0.0, 0.0)
    background(220, 200, 140)

    draw_geometry(geometry)

def create_geometry():
    global geometry

    beginGeometry()

    for i in range(60):        
        x = gauss(400, 200)
        y = gauss(350, 175)
        z = gauss(0, 100)
        
        push_matrix()
        translate(x, y, z)
        sphere(10)
        pop_matrix()

    geometry = endGeometry()

# TODO: this should happen implicitly on module load somehow
run()
