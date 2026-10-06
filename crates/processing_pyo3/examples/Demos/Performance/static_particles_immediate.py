# Static Particles Immediate
#
# 50000 textured sprites at fixed random positions, each drawn with
# its own image() call every frame.
from mewnala import *

sprite = None

npart_total = 50000
part_size = 20

positions = []

fcount = 0
lastm = 0
frate = 0
fint = 3


def setup():
    size(800, 600)
    mode_3d()
    frame_rate(60)

    global sprite
    sprite = load_image("data/sprite.png")

    init_positions()


def draw():
    global fcount, lastm, frate
    background(0.0)

    rotate_y(frame_count * 0.01)

    for n in range(npart_total):
        draw_particle(positions[n])

    fcount += 1
    m = millis()
    if m - lastm > 1000 * fint:
        frate = fcount / fint
        fcount = 0
        lastm = m
        print("fps: " + str(frate))
    window_title("fps: " + str(frate))


def draw_particle(center):
    no_stroke()
    tint(1.0)
    push_matrix()
    translate(0, 0, center.z)
    image(sprite, center.x - part_size / 2, center.y - part_size / 2, part_size, part_size)
    pop_matrix()


def init_positions():
    for n in range(npart_total):
        positions.append(Vec3(random(-500, +500), random(-500, +500), random(-500, +500)))


run()
