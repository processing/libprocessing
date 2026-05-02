from mewnala import *

GRID = 100  # GRID^3 = 1,000,000 particles
SPACING = 1.0
SPIN_PER_FRAME = 0.003

field_obj = None
cube = None
mat = None
spin = None


def setup():
    global field_obj, cube, mat, spin

    size(900, 700)
    mode_3d()

    extent = GRID * SPACING * 0.5
    camera_position(0.0, extent * 0.6, extent * 2.5)
    camera_look_at(0.0, 0.0, 0.0)
    orbit_camera()

    # Three directional R/G/B lights from cardinal axes.
    red = create_directional_light((1.0, 0.0, 0.0), 1000.0)
    red.position(1.0, 0.0, 0.0)
    red.look_at(0.0, 0.0, 0.0)
    green = create_directional_light((0.0, 1.0, 0.0), 1000.0)
    green.position(0.0, 1.0, 0.0)
    green.look_at(0.0, 0.0, 0.0)
    blue = create_directional_light((0.0, 0.0, 1.0), 1000.0)
    blue.position(0.0, 0.0, 1.0)
    blue.look_at(0.0, 0.0, 0.0)

    field_obj = Field(
        geometry=Geometry.grid(GRID, GRID, GRID, SPACING),
        attributes=[Attribute.position(), Attribute.uv(), Attribute.color()],
    )

    # One-shot noise pass to break the regular lattice up. `scale` is the
    # input multiplier applied to position before sampling — at < 1 / SPACING
    # adjacent grid points sample nearly the same noise cell and get nearly
    # identical displacement, leaving the lattice visible. Bumping it past
    # 1 / SPACING breaks the grid.
    jitter = kernel_noise()
    jitter.set(scale=1.0 / SPACING, strength=SPACING * 0.6, time=0.0)
    field_obj.apply(jitter)

    # Color each particle by its lattice u-coord.
    uv_buf = field_obj.buffer(Attribute.uv())
    color_buf = field_obj.buffer(Attribute.color())
    colors = []
    for uv in uv_buf.read():
        c = hsva(uv[0] * 360.0, 0.85, 1.0)
        colors.append([c.r, c.g, c.b, 1.0])
    color_buf.write(colors)

    mat = Material.pbr(albedo=color_buf)

    cube = Geometry.box(0.35, 0.35, 0.35)

    spin = kernel_transform()
    spin.set(rotation=[0.0, 1.0, 0.0, SPIN_PER_FRAME])


def draw():
    background(10, 10, 18)

    use_material(mat)
    draw_field(field_obj, cube)

    field_obj.apply(spin)


run()
