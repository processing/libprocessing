from mewnala import *

p = None
particle = None
mat = None


def setup():
    global p, particle, mat

    size(900, 700)
    mode_3d()

    directional_light((0.95, 0.9, 0.85), 600.0)

    source = Geometry.sphere(5.0, 32, 24)
    p = Particles(
        geometry=source,
        attributes=[Attribute.position(), Attribute.uv(), Attribute.color()],
    )

    uv_buf = p.buffer(Attribute.uv())
    color_buf = p.buffer(Attribute.color())

    colors = []
    for uv in uv_buf.read():
        c = hsva(uv[0] * 360.0, 0.85, 1.0)
        colors.append([c.r, c.g, c.b, 1.0])
    color_buf.write(colors)

    particle = Geometry.sphere(0.18, 10, 8)
    mat = Material.pbr(albedo=color_buf)


def draw():
    camera_position(0.0, 4.0, 18.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(15, 15, 20)

    use_material(mat)
    particles(p, particle)


run()
