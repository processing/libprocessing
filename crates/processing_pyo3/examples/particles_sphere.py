from mewnala import *
from math import cos, sin

from mewnala.mewnala import TRIANGLES

p = None
rest = None


def setup():
    global p
    size(900, 700)
    window_title("Sphere")
    mode_3d()

    rest = Attribute("rest", AttributeFormat.Float3)
    sphere = Geometry.sphere(1.5, 96, 64)
    p = create_particles(
        geometry=sphere,
        attributes=[
            Attribute.position(),
            rest,
        ],
    )
    p.apply(MAP, a=Attribute.position(), out=rest, op=AFFINE, scale=1.0, offset=0.0)


def draw():
    background(6, 8, 14)

    t = elapsed_time
    r = 5.0
    camera_position(cos(t * 0.15) * r, 1.6, sin(t * 0.15) * r)
    camera_look_at(0.0, 0.0, 0.0)

    p.apply(NOISE, scale=0.9, strength=0.01, time=t * 0.3, divergence_free=1)
    particles(p, topology=TRIANGLES)

run()
