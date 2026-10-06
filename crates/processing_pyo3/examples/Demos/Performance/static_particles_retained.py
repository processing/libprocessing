# Static Particles Retained
#
# 50000 textured sprites at fixed random positions, built once into a
# single geometry and drawn with one call per frame.
from mewnala import *

particles_geom = None
sprite = None

npart_total = 50000
part_size = 20

fcount = 0
lastm = 0
frate = 0
fint = 3


def setup():
    global particles_geom, sprite
    size(800, 600)
    mode_3d()
    frame_rate(60)

    sprite = load_image("data/sprite.png")
    particles_geom = create_geometry(topology=TRIANGLES)

    for n in range(npart_total):
        cx = random(-500, +500)
        cy = random(-500, +500)
        cz = random(-500, +500)

        # One textured quad (two triangles) per particle
        base = 4 * n
        particles_geom.uv(0, 0)
        particles_geom.vertex(cx - part_size / 2, cy - part_size / 2, cz)
        particles_geom.uv(1, 0)
        particles_geom.vertex(cx + part_size / 2, cy - part_size / 2, cz)
        particles_geom.uv(1, 1)
        particles_geom.vertex(cx + part_size / 2, cy + part_size / 2, cz)
        particles_geom.uv(0, 1)
        particles_geom.vertex(cx - part_size / 2, cy + part_size / 2, cz)
        for i in (0, 1, 2, 0, 2, 3):
            particles_geom.index(base + i)


def draw():
    global fcount, lastm, frate
    background(0.0)

    rotate_y(frame_count * 0.01)

    no_stroke()
    tint(1.0)
    texture(sprite)
    draw_geometry(particles_geom)
    no_texture()

    fcount += 1
    m = millis()
    if m - lastm > 1000 * fint:
        frate = fcount / fint
        fcount = 0
        lastm = m
        print("fps: " + str(frate))
    window_title("fps: " + str(frate))


run()
