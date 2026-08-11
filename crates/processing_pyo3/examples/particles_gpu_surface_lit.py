from mewnala import *
from math import cos, sin

NX, NY = 200, 200
EXTENT = 30.0

p = None
gen = None


def setup():
    global p, gen
    size(900, 700)
    window_title("GPU surface — color + normals generated on the GPU")
    mode_3d()

    p = create_particles(
        capacity=NX * NY,
        attributes=[Attribute.position(), Attribute.color(), Attribute.normal()],
    )
    idx = p.index_buffer((NX - 1) * (NY - 1) * 6)
    gen = create_compute(load_shader("shaders/gen_colored_surface.wesl"))
    gen.set(indices=idx)


def draw():
    background(6, 8, 14)

    t = elapsed_time
    d = EXTENT * 1.3
    camera_position(cos(t * 0.08) * d, EXTENT * 0.6, sin(t * 0.08) * d)
    camera_look_at(0.0, 0.0, 0.0)

    gen.set(nx=NX, ny=NY, time=t, extent=EXTENT)
    p.apply(gen)

    particles(p, topology=TRIANGLES)


run()
