# Dynamic Particles Retained
#
# 10000 fading sprites emitted from the mouse. The particles live in a GPU
# particle system (the retained equivalent of the PShape group): the CPU
# updates their positions and tints and uploads them once per frame.
from mewnala import *

particle_system = None
quad = None
sprite = None
mat = None

npart_total = 10000
npart_per_frame = 25
speed = 1.0
gravity = 0.05
part_size = 20

part_lifetime = 0
positions = []
velocities = []
lifetimes = []
tints = []

fcount = 0
lastm = 0
frate = 0
fint = 3


def setup():
    global particle_system, quad, sprite, mat, part_lifetime
    size(640, 480)
    mode_3d()
    frame_rate(120)

    sprite = load_image("data/sprite.png")

    particle_system = create_particles(npart_total, attributes=[Attribute.position(), Attribute.color()])
    # Each particle's tint comes from the color buffer (the setTint of the original)
    mat = create_material(unlit=True, albedo=particle_system.buffer("color"))

    # The sprite quad every particle is drawn with
    quad = create_geometry(topology=TRIANGLES)
    quad.uv(0, 0)
    quad.vertex(-part_size / 2, -part_size / 2, 0)
    quad.uv(1, 0)
    quad.vertex(+part_size / 2, -part_size / 2, 0)
    quad.uv(1, 1)
    quad.vertex(+part_size / 2, +part_size / 2, 0)
    quad.uv(0, 1)
    quad.vertex(-part_size / 2, +part_size / 2, 0)
    for i in (0, 1, 2, 0, 2, 3):
        quad.index(i)

    part_lifetime = npart_total // npart_per_frame
    init_positions()
    init_velocities()
    init_lifetimes()


def draw():
    global fcount, lastm, frate
    background(0.0)

    for n in range(npart_total):
        lifetimes[n] += 1
        if lifetimes[n] == part_lifetime:
            lifetimes[n] = 0

        if 0 <= lifetimes[n]:
            opacity = 1.0 - lifetimes[n] / part_lifetime
            tints[n] = [1.0, 1.0, 1.0, opacity]

            if lifetimes[n] == 0:
                # Re-spawn dead particle (the 3D origin is the canvas center, y up)
                positions[n] = [mouse_x - width / 2, height / 2 - mouse_y, 0.0]
                angle = random(0, TWO_PI)
                s = random(0.5 * speed, 0.5 * speed)
                velocities[n].x = s * cos(angle)
                velocities[n].y = s * sin(angle)
            else:
                positions[n][0] += velocities[n].x
                positions[n][1] += velocities[n].y
                velocities[n].y -= gravity
        else:
            tints[n] = [0.0, 0.0, 0.0, 0.0]

    particle_system.buffer("position").write(positions)
    particle_system.buffer("color").write(tints)

    no_stroke()
    material(mat)
    texture(sprite)
    particles(particle_system, quad)
    no_texture()

    fcount += 1
    m = millis()
    if m - lastm > 1000 * fint:
        frate = fcount / fint
        fcount = 0
        lastm = m
        print("fps: " + str(frate))
    window_title("fps: " + str(frate))


def init_positions():
    for n in range(npart_total):
        # Parked far away until the particle is born
        positions.append([0.0, 0.0, -100000.0])
        tints.append([0.0, 0.0, 0.0, 0.0])


def init_velocities():
    for n in range(npart_total):
        velocities.append(Vec2())


def init_lifetimes():
    # Initializing particles with negative lifetimes so they are added
    # progressively into the screen during the first frames of the sketch
    t = -1
    for n in range(npart_total):
        if n % npart_per_frame == 0:
            t += 1
        lifetimes.append(-t)


run()
