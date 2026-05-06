from mewnala import *

GRID = 150
SPACING = 1.0
SPIN_PER_FRAME = 0.003

p = None
cube = None
spin = None


def setup():
    global p, cube, spin

    size(900, 700)
    mode_3d()

    extent = GRID * SPACING * 0.5
    camera_position(0.0, extent * 0.6, extent * 2.5)
    camera_look_at(0.0, 0.0, 0.0)
    orbit_camera()

    directional_light((1.0, 0.0, 0.0), 1000.0, position=Vec3.X, look_at=Vec3.ZERO)
    directional_light((0.0, 1.0, 0.0), 1000.0, position=Vec3.Y, look_at=Vec3.ZERO)
    directional_light((0.0, 0.0, 1.0), 1000.0, position=Vec3.Z, look_at=Vec3.ZERO)

    p = Particles(
        geometry=Geometry.grid(GRID, GRID, GRID, SPACING),
        attributes=[Attribute.position(), Attribute.uv(), Attribute.color()],
    )

    p.apply(kernel_noise(), scale=1.0 / SPACING, strength=SPACING * 0.6)

    color_buf = p.buffer(Attribute.color())
    color_buf.write([
        [c.r, c.g, c.b, 1.0]
        for uv in p.buffer(Attribute.uv()).read()
        for c in [hsva(uv[0] * 360.0, 0.85, 1.0)]
    ])

    fill(color_buf)
    cube = Geometry.box(0.35, 0.35, 0.35)
    spin = kernel_transform()


def draw():
    background(10, 10, 18)
    particles(p, cube)
    p.apply(spin, rotation_axis=Vec3.Y, rotation_angle=SPIN_PER_FRAME)

run()
