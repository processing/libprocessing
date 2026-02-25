import math
from processing import *

gltf = None
duck_geo = None
duck_mat = None
light = None
frame = 0

def setup():
    global gltf, duck_geo, duck_mat, light
    size(800, 600)

    gltf = load_gltf("gltf/Duck.glb")
    duck_geo = gltf.geometry("LOD3spShape")
    duck_mat = gltf.material("blinn3-fx")

    mode_3d()
    gltf.camera(0)
    light = gltf.light(0)

def draw():
    global frame
    t = frame * 0.02

    radius = 1.5
    lx = math.cos(t) * radius
    ly = 1.5
    lz = math.sin(t) * radius
    light.position(lx, ly, lz)
    light.look_at(0.0, 0.8, 0.0)

    r = math.sin(t * 8.0) * 0.5 + 0.5
    g = math.sin(t * 8.0 + 2.0) * 0.5 + 0.5
    b = math.sin(t * 8.0 + 4.0) * 0.5 + 0.5
    duck_mat.set_float4("base_color", r, g, b, 1.0)

    background(25)
    use_material(duck_mat)
    draw_geometry(duck_geo)

    frame += 1


# TODO: this should happen implicitly on module load somehow
run()
