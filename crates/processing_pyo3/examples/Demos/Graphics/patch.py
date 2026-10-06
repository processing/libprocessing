# Bezier patch By Maritus Watz:
# http://www.openprocessing.org/sketch/57709
# Normal calculation added by Andres Colubri
# Direct port of sample code by Paul Bourke.
# Original code: http://paulbourke.net/geometry/bezier/
#
# Press space to build a new random patch.
from mewnala import *

ni = 4
nj = 5
RESI = ni * 10
RESJ = nj * 10
outp = []
inp = []
normp = []
auto_normals = False
patch = None


def setup():
    size(1024, 768)
    mode_3d()
    # lights()
    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)
    build()


def draw():
    background(1.0)
    scale(0.9)
    rotate_y(remap(mouse_x, 0, width, -PI, PI))
    rotate_x(remap(mouse_y, 0, height, -PI, PI))

    no_stroke()
    fill(1.0)
    draw_geometry(patch)


def key_pressed():
    if key == " ":
        build()
    save_frame("bezPatch.png")


def build():
    global outp, normp, inp, patch

    outp = []
    normp = []
    inp = []
    uitang = Vec3()
    ujtang = Vec3()

    for i in range(ni + 1):
        row = []
        for j in range(nj + 1):
            row.append(Vec3(i, j, random(-3, 3)))
        inp.append(row)

    for i in range(RESI):
        mui = i / (RESI - 1)
        out_row = []
        norm_row = []
        for j in range(RESJ):
            muj = j / (RESJ - 1)
            out = Vec3()
            uitang.set(0, 0, 0)
            ujtang.set(0, 0, 0)
            for ki in range(ni + 1):
                bi = bezier_blend(ki, mui, ni)
                dbi = d_bezier_blend(ki, mui, ni)
                for kj in range(nj + 1):
                    bj = bezier_blend(kj, muj, nj)
                    dbj = d_bezier_blend(kj, muj, nj)
                    out.x += inp[ki][kj].x * bi * bj
                    out.y += inp[ki][kj].y * bi * bj
                    out.z += inp[ki][kj].z * bi * bj

                    uitang.x += inp[ki][kj].x * dbi * bj
                    uitang.y += inp[ki][kj].y * dbi * bj
                    uitang.z += inp[ki][kj].z * dbi * bj

                    ujtang.x += inp[ki][kj].x * bi * dbj
                    ujtang.y += inp[ki][kj].y * bi * dbj
                    ujtang.z += inp[ki][kj].z * bi * dbj
            out.add(Vec3(-ni / 2, -nj / 2, 0))
            out.mult(100)
            out_row.append(out)

            uitang.normalize()
            ujtang.normalize()
            norm_row.append(uitang.cross(ujtang))
        outp.append(out_row)
        normp.append(norm_row)

    # The quad strips of the original become one indexed triangle mesh
    patch = create_geometry(topology=TRIANGLES)
    for i in range(RESI):
        for j in range(RESJ):
            if not auto_normals:
                patch.normal(normp[i][j].x, normp[i][j].y, normp[i][j].z)
            patch.vertex(outp[i][j].x, outp[i][j].y, outp[i][j].z)
    for i in range(RESI - 1):
        for j in range(RESJ - 1):
            a = i * RESJ + j
            b = (i + 1) * RESJ + j
            patch.index(a)
            patch.index(b)
            patch.index(a + 1)
            patch.index(a + 1)
            patch.index(b)
            patch.index(b + 1)


def bezier_blend(k, mu, n):
    blend = 1

    nn = n
    kn = k
    nkn = n - k

    while nn >= 1:
        blend *= nn
        nn -= 1
        if kn > 1:
            blend /= kn
            kn -= 1
        if nkn > 1:
            blend /= nkn
            nkn -= 1
    if k > 0:
        blend *= pow(mu, k)
    if n - k > 0:
        blend *= pow(1 - mu, n - k)

    return blend


def d_bezier_blend(k, mu, n):
    dblendf = 1

    nn = n
    kn = k
    nkn = n - k

    while nn >= 1:
        dblendf *= nn
        nn -= 1
        if kn > 1:
            dblendf /= kn
            kn -= 1
        if nkn > 1:
            dblendf /= nkn
            nkn -= 1

    fk = 1
    dk = 0
    fnk = 1
    dnk = 0
    if k > 0:
        fk = pow(mu, k)
        dk = k * pow(mu, k - 1)
    if n - k > 0:
        fnk = pow(1 - mu, n - k)
        dnk = (k - n) * pow(1 - mu, n - k - 1)
    dblendf *= dk * fnk + fk * dnk

    return dblendf


run()
