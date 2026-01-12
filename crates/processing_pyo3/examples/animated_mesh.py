from processing import *
from math import sin, cos

geometry = None
grid_size = 20 
spacing = 10.0
offset = (grid_size * spacing) / 2.0
time = 0.0

def setup():
    global geometry
    size(800, 600)
    mode_3d()
    geometry = Geometry()
    for z in range(grid_size):
        for x in range(grid_size):
            px = x * spacing - offset
            pz = z * spacing - offset
            geometry.color(x/grid_size, 0.5, z/grid_size, 1.0)
            geometry.normal(0.0, 1.0, 0.0)
            geometry.vertex(px, 0.0, pz)

    for z in range(grid_size-1):
        for x in range(grid_size-1):
            tl = z * grid_size + x
            tr = tl + 1
            bl = (z + 1) * grid_size + x
            br = bl + 1

            geometry.index(tl)
            geometry.index(bl)
            geometry.index(tr)

            geometry.index(tr)
            geometry.index(bl)
            geometry.index(br)


def draw():
    global geometry
    global grid_size
    global offset
    global spacing
    global time

    camera_position(150.0, 150.0, 150.0)
    camera_look_at( 0.0, 0.0, 0.0)
    background(220, 200, 140)

    for z in range(grid_size):
        for x in range(grid_size):
            idx = int(z * grid_size + x)
            px = x * spacing - offset
            pz = z * spacing - offset
            wave = sin(px * 0.1 + time) * cos(pz * 0.1 + time) * 20.0
            geometry.set_vertex(idx, px, wave, pz)

    draw_geometry(geometry)

    time += 0.05


# TODO: this should happen implicitly on module load somehow
run()
