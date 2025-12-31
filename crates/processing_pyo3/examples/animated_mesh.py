from processing import *

mesh = None
grid_size = 20 
spacing = 10.0

def setup():
    size(800, 600)
    mode_3d()
    global mesh = geometry_create(TriangleList)

def draw():
    # TODO: maybe all of this should be in `setup()`
    global grid_size

    for z in range(grid_size):
        for x in range(grid_size):
            px = x * spacing - offset
            pz = z * spacing - offset
            geometry_color(mesh, x/grid_size, 0.5, z/grid_size, 1.0)
            geometry_normal(mesh, 0.0,1.0,0.0)
            geometry_vertex(mesh, px, 0.0, pz)
    for z in range(grid_size-1):
        for x in range(grid_size-1):
            tl = z * grid_size + x
            tr = tl + 1
            bl = (z + 1) * grid_size + x
            br = bl + 1

            geometry_index(mesh, tl)
            geometry_index(mesh, bl)
            geometry_index(mesh, tr)

            geometry_index(mesh, tr)
            geometry_index(mesh, bl)
            geometry_index(mesh, br)

        


# TODO: this should happen implicitly on module load somehow
run()
