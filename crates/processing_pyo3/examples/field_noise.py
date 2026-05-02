from mewnala import *

field_obj = None
particle = None
mat = None
noise = None


def setup():
    global field_obj, particle, mat, noise

    size(900, 700)
    mode_3d()

    create_directional_light((0.95, 0.9, 0.85), 200.0)

    # Seed positions from a sphere mesh; noise will jitter them around their
    # initial sphere shape over time.
    source = Geometry.sphere(5.0, 32, 24)
    field_obj = Field(
        geometry=source,
        attributes=[Attribute.position(), Attribute.uv(), Attribute.color()],
    )

    uv_buf = field_obj.buffer(Attribute.uv())
    color_buf = field_obj.buffer(Attribute.color())

    colors = []
    for uv in uv_buf.read():
        c = hsva(uv[0] * 360.0, 0.85, 1.0)
        colors.append([c.r, c.g, c.b, 1.0])
    color_buf.write(colors)

    particle = Geometry.sphere(0.18, 10, 8)
    mat = Material.field_pbr(color_buf)
    noise = kernel_noise()


def draw():
    camera_position(0.0, 4.0, 18.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(15, 15, 20)

    use_material(mat)
    draw_field(field_obj, particle)

    noise.set(scale=0.25, strength=0.02, time=elapsed_time * 0.5)
    field_obj.apply(noise)


run()
