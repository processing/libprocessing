# Dynamic Particles Immediate
#
# 10000 fading sprites emitted from the mouse, each drawn with its own
# tinted image() call every frame.
from mewnala import *

sprite = None

npart_total = 10000
npart_per_frame = 25
speed = 1.0
gravity = 0.05
part_size = 20

part_lifetime = 0
positions = []
velocities = []
lifetimes = []

fcount = 0
lastm = 0
frate = 0
fint = 3


def setup():
    global sprite, part_lifetime
    size(640, 480)
    mode_3d()
    frame_rate(120)

    sprite = load_image("data/sprite.png")

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

            if lifetimes[n] == 0:
                # Re-spawn dead particle (the 3D origin is the canvas center, y up)
                positions[n].x = mouse_x - width / 2
                positions[n].y = height / 2 - mouse_y

                angle = random(0, TWO_PI)
                s = random(0.5 * speed, 0.5 * speed)
                velocities[n].x = s * cos(angle)
                velocities[n].y = s * sin(angle)
            else:
                positions[n].x += velocities[n].x
                positions[n].y += velocities[n].y

                velocities[n].y -= gravity
            draw_particle(positions[n], opacity)

    fcount += 1
    m = millis()
    if m - lastm > 1000 * fint:
        frate = fcount / fint
        fcount = 0
        lastm = m
        print("fps: " + str(frate))
    window_title("fps: " + str(frate))


def draw_particle(center, opacity):
    no_stroke()
    tint(1.0, opacity)
    image(sprite, center.x - part_size / 2, center.y - part_size / 2, part_size, part_size)


def init_positions():
    for n in range(npart_total):
        positions.append(Vec2())


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
