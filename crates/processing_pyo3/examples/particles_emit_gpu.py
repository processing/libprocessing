from mewnala import *
import math

p = None
particle = None
mat = None
spawn = None
motion = None

CAPACITY = 40000
BURST = 120
DT = 1.0 / 60.0
TTL = 2.5
GRAVITY = 9.8
SPEED = 5.0


def setup():
    global p, particle, mat, spawn, motion

    size(900, 700)
    mode_3d()

    directional_light((0.95, 0.9, 0.85), 800.0)

    particle = Geometry.sphere(0.12, 8, 6)

    # Attributes a compute shader binds must exist when its bind group is built,
    # so declare them (all built-in here). `velocity`/`age` are built-ins now, so
    # no custom `Attribute(...)` is needed.
    p = create_particles(
        capacity=CAPACITY,
        attributes=[
            Attribute.position(),
            Attribute.velocity(),
            Attribute.color(),
            Attribute.scale(),
            Attribute.age(),
            Attribute.life(),
        ],
    )

    mat = create_material(albedo=p.buffer("color"))

    spawn = create_compute(load_shader("shaders/particles_emit_gpu_spawn.wesl"))
    motion = create_compute(load_shader("shaders/particles_emit_gpu_motion.wesl"))


def draw():
    camera_position(0.0, 4.0, 16.0)
    camera_look_at(0.0, 2.0, 0.0)
    background(10, 10, 18)

    material(mat)
    particles(p, particle)

    t = elapsed_time
    sx = math.cos(t) * 0.4
    sz = math.sin(t) * 0.4
    spawn.set(pos=[sx, 7.0, sz, 0.0], speed=[SPEED, 0.0, 0.0, 0.0])
    p.emit_gpu(BURST, spawn)

    motion.set(dt=DT, ttl=TTL, gravity=GRAVITY)
    p.apply(motion)


run()
