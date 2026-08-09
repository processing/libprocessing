from mewnala import *

p = None
sphere = None
mat = None
spin = None


def setup():
    global p, sphere, mat, spin

    size(900, 700)
    mode_3d()

    directional_light((0.9, 0.85, 0.8), 300.0)

    sphere = Geometry.sphere(0.25, 12, 8)

    capacity = 1000
    positions = []
    for x in range(10):
        for y in range(10):
            for z in range(10):
                positions.extend([x - 4.5, y - 4.5, z - 4.5])

    p = create_particles(capacity=capacity, attributes=[Attribute.position()])
    pos_buf = p.buffer("position")
    pos_buf.write(positions)

    mat = create_material(roughness=0.4)
    spin = create_compute(load_shader("shaders/particles_animated_spin.wesl"))


def draw():
    camera_position(0.0, 8.0, 25.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(15, 15, 20)
    fill(230, 128, 75)

    material(mat)
    particles(p, sphere)

    spin.set(dt=0.01)
    p.apply(spin)


run()
