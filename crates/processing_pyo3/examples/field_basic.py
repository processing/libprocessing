from mewnala import *

field_obj = None
particle = None
mat = None


def setup():
    global field_obj, particle, mat

    size(900, 700)
    mode_3d()

    create_directional_light((0.95, 0.9, 0.85), 600.0)

    # Source mesh whose vertices become particle positions; uvs come along for
    # free and we use them to color each particle.
    source = Geometry.sphere(5.0, 32, 24)
    field_obj = Field(
        geometry=source,
        attributes=[Attribute.position(), Attribute.uv(), Attribute.color()],
    )

    # Read uvs back, build per-particle colors, write to color PBuffer.
    color_buf = field_obj.pbuffer(Attribute.color())
    uv_buf = field_obj.pbuffer(Attribute.uv())
    colors = []
    for uv in uv_buf.read():
        u = uv[0]
        h = u * 6.0
        c = h - int(h)
        if h < 1:
            colors.append([1.0, c, 0.0, 1.0])
        elif h < 2:
            colors.append([1.0 - c, 1.0, 0.0, 1.0])
        elif h < 3:
            colors.append([0.0, 1.0, c, 1.0])
        elif h < 4:
            colors.append([0.0, 1.0 - c, 1.0, 1.0])
        elif h < 5:
            colors.append([c, 0.0, 1.0, 1.0])
        else:
            colors.append([1.0, 0.0, 1.0 - c, 1.0])
    color_buf.write(colors)

    particle = Geometry.sphere(0.18, 10, 8)
    mat = Material.field_pbr(color_buf)


def draw():
    camera_position(0.0, 4.0, 18.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(15, 15, 20)

    use_material(mat)
    draw_field(field_obj, particle)


run()
