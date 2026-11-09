from processing_py import *

p = None
particle = None
mat = None
sun = None


def setup():
    global p, particle, mat, sun

    size(900, 700)
    mode_3d()

    sun = create_directional_light((0.95, 0.9, 0.85), 600.0)

    source = Geometry.sphere(5.0, 32, 24)
    p = create_particles(
        source,
        attributes=[Attribute.position(), Attribute.uv(), Attribute.color()],
    )

    uv_buf = p.buffer("uv")
    color_buf = p.buffer("color")

    colors = []
    for uv in uv_buf.read():
        c = hsva(uv[0] * 360.0, 0.85, 1.0)
        colors.append([c.r, c.g, c.b, 1.0])
    color_buf.write(colors)

    particle = Geometry.sphere(0.18, 10, 8)
    mat = create_material(albedo=color_buf)


def draw():
    camera_position(0.0, 4.0, 18.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(15, 15, 20)
    light(sun)

    material(mat)
    particles(p, particle)


run()
