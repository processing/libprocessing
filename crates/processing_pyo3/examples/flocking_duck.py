# Flocking inside a duck: the GPU boids from flocking_gpu.py, seeded from
# the vertices of the Duck glTF mesh. Each boid remembers its spawn vertex
# in a `home` attribute; a homing force that is negligible near home but
# grows quadratically with distance lets the boids swirl and flock locally
# while the swarm as a whole never loses the duck's shape.
from mewnala import *
from math import cos, sin
from random import uniform

DT = 1.0 / 60.0

# Pass 1: Reynolds' three rules plus the tether. Every boid reads the whole
# flock's state and writes only its steering force, so no boid ever sees a
# half-updated neighbor.

# Pass 2: integrate the steering force and point each instanced boid along
# its velocity. No wrapping — the tether is the only containment needed.

p = None
boid = None
mat = None
flock_pass = None
integrate_pass = None
center = None
extent = 0.0
max_speed = 0.0
boid_count = 0
title_last_time = 0.0
title_last_frame = 0


# Two triangles folded slightly along the nose-tail spine, like a paper
# boid pointing down +Z. The fold keeps the boid visible edge-on and gives
# each wing its own normal, so the flock glints as it banks.
def boid_geometry(half_width, length, droop):
    g = create_geometry()
    n = (half_width * half_width + droop * droop) ** 0.5
    nose = (0.0, 0.0, length * 0.5)
    tail = (0.0, 0.0, -length * 0.5)
    g.normal(-droop / n, half_width / n, 0.0)
    g.vertex(*nose)
    g.vertex(-half_width, -droop, -length * 0.5)
    g.vertex(*tail)
    g.normal(droop / n, half_width / n, 0.0)
    g.vertex(*nose)
    g.vertex(*tail)
    g.vertex(half_width, -droop, -length * 0.5)
    for i in range(6):
        g.index(i)
    return g


def setup():
    global p, boid, mat, flock_pass, integrate_pass, center, extent, max_speed, boid_count

    size(900, 700)
    mode_3d()

    directional_light((0.95, 0.9, 0.85), 800.0)

    gltf = load_gltf("gltf/Duck.glb")
    duck = gltf.geometry("LOD3spShape")

    p = create_particles(
        geometry=duck,
        attributes=[
            Attribute.position(),
            Attribute.rotation(),
            Attribute.color(),
            Attribute.velocity(),
            Attribute("home", AttributeFormat.Float3),
            Attribute("steer", AttributeFormat.Float3),
        ],
    )

    # The duck's vertices become the boids' homes. Every tuning constant is
    # derived from the mesh's bounding box, so the sketch doesn't care what
    # units the model was authored in.
    homes = p.buffer("position").read()
    boid_count = len(homes)
    window_title(f"GPU Flocking Duck — {boid_count:,} boids")
    lo = [min(v[i] for v in homes) for i in range(3)]
    hi = [max(v[i] for v in homes) for i in range(3)]
    center = [(lo[i] + hi[i]) * 0.5 for i in range(3)]
    extent = sum((hi[i] - lo[i]) ** 2 for i in range(3)) ** 0.5
    max_speed = 0.15 * extent

    p.buffer("home").write(homes)

    velocities = []
    rotations = []
    colors = []
    for _ in homes:
        velocities.append([uniform(-1.0, 1.0) * max_speed * 0.4 for _ in range(3)])
        rotations.append([0.0, 0.0, 0.0, 1.0])
        c = hsva(uniform(38.0, 58.0), 0.85, 1.0)
        colors.append([c.r, c.g, c.b, 1.0])

    p.buffer("velocity").write(velocities)
    p.buffer("rotation").write(rotations)
    color_buf = p.buffer("color")
    color_buf.write(colors)

    s = 0.008 * extent
    boid = boid_geometry(1.2 * s, 3.5 * s, 0.4 * s)
    mat = create_material(albedo=color_buf)

    flock_pass = create_compute(load_shader("shaders/flocking_duck_flock.wesl"))
    integrate_pass = create_compute(load_shader("shaders/flocking_duck_integrate.wesl"))


def draw():
    global title_last_time, title_last_frame

    title_elapsed = elapsed_time - title_last_time
    if title_elapsed >= 0.5:
        fps = (frame_count - title_last_frame) / title_elapsed
        window_title(f"GPU Flocking Duck — {boid_count:,} boids — {fps:.0f} FPS")
        title_last_time = elapsed_time
        title_last_frame = frame_count

    t = elapsed_time * 0.2
    r = extent * 1.1
    camera_position(center[0] + cos(t) * r, center[1] + extent * 0.35, center[2] + sin(t) * r)
    camera_look_at(center[0], center[1], center[2])
    background(10, 12, 18)

    material(mat)
    particles(p, boid)

    flock_pass.set(
        neighbor_dist=0.06 * extent,
        separation_dist=0.03 * extent,
        max_speed=max_speed,
        max_force=2.0 * max_speed,
        home_radius=0.04 * extent,
    )
    p.apply(flock_pass)

    integrate_pass.set(dt=DT, max_speed=max_speed)
    p.apply(integrate_pass)


run()
