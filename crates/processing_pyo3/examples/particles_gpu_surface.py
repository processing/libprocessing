from mewnala import *
from math import cos, sin

NX, NY = 160, 160
EXTENT = 24.0

p = None
gen = None


def setup():
    global p, gen
    size(900, 700)
    window_title("gpu index")
    mode_3d()

    p = create_particles(capacity=NX * NY, attributes=[Attribute.position()])
    idx = p.index_buffer((NX - 1) * (NY - 1) * 6)
    gen = create_compute(load_shader("shaders/gen_surface.wesl"))
    gen.set(indices=idx)


def draw():
    background(6, 8, 14)

    t = elapsed_time
    d = EXTENT * 1.4
    camera_position(cos(t * 0.1) * d, EXTENT * 0.7, sin(t * 0.1) * d)
    camera_look_at(0.0, 0.0, 0.0)

    gen.set(nx=NX, ny=NY, time=t, extent=EXTENT)
    p.apply(gen)

    particles(p, topology=TRIANGLES)


run()
