# Planets, by Andres Colubri
#
# Sun and mercury textures from http://planetpixelemporium.com
# Star field picture from http://www.galacticimages.com/
from mewnala import *

starfield = None

suntex = None
surftex1 = None
cloudtex = None
surftex2 = None

sun_light = None
planet_light = None


def setup():
    global starfield, suntex, surftex1, surftex2, sun_light, planet_light
    size(1024, 768)
    mode_3d()

    starfield = load_image("data/starfield.jpg")
    suntex = load_image("data/sun.jpg")
    surftex1 = load_image("data/planet.jpg")
    surftex2 = load_image("data/mercury.jpg")

    # The clouds texture of the original is generated with the Perlin class
    # below (seamless 3D noise mapped onto the sphere); that code is disabled
    # upstream as well.

    no_stroke()
    fill(1.0)

    # pointLight(255, 255, 255, 0, 0, 0) at the sun's position, and
    # pointLight(255, 255, 255, 0, 0, -150) at the screen corner for planet1
    sun_light = point_light((1.0, 1.0, 1.0), 1000000.0, 1200.0, 1.0, position=(0, 0, -300))
    planet_light = point_light((1.0, 1.0, 1.0), 1000000.0, 1200.0, 1.0, position=(-width / 2, height / 2, -150))


def draw():
    # The star field is drawn as the background so it never occludes the 3D objects
    background(starfield)

    push_matrix()
    translate(0, 0, -300)

    # The sun is drawn before the lights are turned on, so it is unlit
    push_matrix()
    rotate_y(PI * frame_count / 500)
    unlit()
    texture(suntex)
    sphere(150, 40, 40)
    no_texture()
    pop_matrix()

    roughness(0.6)
    rotate_y(PI * frame_count / 300)
    translate(0, 0, 300)

    texture(surftex2)
    sphere(50, 40, 40)
    no_texture()

    pop_matrix()

    translate(0.25 * width, -0.1 * height, 50)
    texture(surftex1)
    sphere(150, 40, 40)
    no_texture()


# Implementation of 1D, 2D, and 3D Perlin noise. Based on the
# C code by Paul Bourke:
# http://local.wasp.uwa.edu.au/~pbourke/texture_colour/perlin/
class Perlin:
    def __init__(self):
        self.B = 0x100
        self.BM = 0xFF
        self.N = 0x1000
        self.NP = 12
        self.NM = 0xFFF

        n = self.B + self.B + 2
        self.p = [0] * n
        self.g3 = []
        self.g2 = []
        self.g1 = [0.0] * n
        for i in range(n):
            self.g3.append([0.0, 0.0, 0.0])
            self.g2.append([0.0, 0.0])

        self.init()

    def normalize2(self, v):
        s = sqrt(v[0] * v[0] + v[1] * v[1])
        v[0] = v[0] / s
        v[1] = v[1] / s

    def normalize3(self, v):
        s = sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
        v[0] = v[0] / s
        v[1] = v[1] / s
        v[2] = v[2] / s

    def s_curve(self, t):
        return t * t * (3.0 - 2.0 * t)

    def at2(self, q, rx, ry):
        return rx * q[0] + ry * q[1]

    def at3(self, q, rx, ry, rz):
        return rx * q[0] + ry * q[1] + rz * q[2]

    def init(self):
        B = self.B
        for i in range(B):
            self.p[i] = i
            self.g1[i] = (random(B + B) - B) / B

            for j in range(2):
                self.g2[i][j] = (random(B + B) - B) / B
            self.normalize2(self.g2[i])

            for j in range(3):
                self.g3[i][j] = (random(B + B) - B) / B
            self.normalize3(self.g3[i])

        i = B - 1
        while 0 < i:
            k = self.p[i]
            j = int(random(B))
            self.p[i] = self.p[j]
            self.p[j] = k
            i -= 1

        for i in range(B + 2):
            self.p[B + i] = self.p[i]
            self.g1[B + i] = self.g1[i]
            for j in range(2):
                self.g2[B + i][j] = self.g2[i][j]
            for j in range(3):
                self.g3[B + i][j] = self.g3[i][j]

    def noise1(self, vec):
        t = vec[0] + self.N
        bx0 = int(t) & self.BM
        bx1 = (bx0 + 1) & self.BM
        rx0 = t - int(t)
        rx1 = rx0 - 1.0

        sx = self.s_curve(rx0)
        u = rx0 * self.g1[self.p[bx0]]
        v = rx1 * self.g1[self.p[bx1]]

        return lerp(u, v, sx)

    def noise2(self, vec):
        t = vec[0] + self.N
        bx0 = int(t) & self.BM
        bx1 = (bx0 + 1) & self.BM
        rx0 = t - int(t)
        rx1 = rx0 - 1.0

        t = vec[1] + self.N
        by0 = int(t) & self.BM
        by1 = (by0 + 1) & self.BM
        ry0 = t - int(t)
        ry1 = ry0 - 1.0

        i = self.p[bx0]
        j = self.p[bx1]

        b00 = self.p[i + by0]
        b10 = self.p[j + by0]
        b01 = self.p[i + by1]
        b11 = self.p[j + by1]

        sx = self.s_curve(rx0)
        sy = self.s_curve(ry0)

        q = self.g2[b00]
        u = self.at2(q, rx0, ry0)
        q = self.g2[b10]
        v = self.at2(q, rx1, ry0)
        a = lerp(u, v, sx)

        q = self.g2[b01]
        u = self.at2(q, rx0, ry1)
        q = self.g2[b11]
        v = self.at2(q, rx1, ry1)
        b = lerp(u, v, sx)

        return lerp(a, b, sy)

    def noise3(self, vec):
        t = vec[0] + self.N
        bx0 = int(t) & self.BM
        bx1 = (bx0 + 1) & self.BM
        rx0 = t - int(t)
        rx1 = rx0 - 1.0

        t = vec[1] + self.N
        by0 = int(t) & self.BM
        by1 = (by0 + 1) & self.BM
        ry0 = t - int(t)
        ry1 = ry0 - 1.0

        t = vec[2] + self.N
        bz0 = int(t) & self.BM
        bz1 = (bz0 + 1) & self.BM
        rz0 = t - int(t)
        rz1 = rz0 - 1.0

        i = self.p[bx0]
        j = self.p[bx1]

        b00 = self.p[i + by0]
        b10 = self.p[j + by0]
        b01 = self.p[i + by1]
        b11 = self.p[j + by1]

        t = self.s_curve(rx0)
        sy = self.s_curve(ry0)
        sz = self.s_curve(rz0)

        q = self.g3[b00 + bz0]
        u = self.at3(q, rx0, ry0, rz0)
        q = self.g3[b10 + bz0]
        v = self.at3(q, rx1, ry0, rz0)
        a = lerp(u, v, t)

        q = self.g3[b01 + bz0]
        u = self.at3(q, rx0, ry1, rz0)
        q = self.g3[b11 + bz0]
        v = self.at3(q, rx1, ry1, rz0)
        b = lerp(u, v, t)

        c = lerp(a, b, sy)

        q = self.g3[b00 + bz1]
        u = self.at3(q, rx0, ry0, rz1)
        q = self.g3[b10 + bz1]
        v = self.at3(q, rx1, ry0, rz1)
        a = lerp(u, v, t)

        q = self.g3[b01 + bz1]
        u = self.at3(q, rx0, ry1, rz1)
        q = self.g3[b11 + bz1]
        v = self.at3(q, rx1, ry1, rz1)
        b = lerp(u, v, t)

        d = lerp(a, b, sy)

        return lerp(c, d, sz)

    # In what follows "nalpha" is the weight when the sum is formed.
    # Typically it is 2, as this approaches 1 the function is noisier.
    # "nbeta" is the harmonic scaling/spacing, typically 2. n is the
    # number of harmonics added up in the final result. Higher number
    # results in more detailed noise.

    def noise_1d(self, x, nalpha, nbeta, n):
        total = 0
        v = [x]
        nscale = 1

        for i in range(n):
            val = self.noise1(v)
            total += val / nscale
            nscale *= nalpha
            v[0] *= nbeta
        return total

    def noise_2d(self, x, y, nalpha, nbeta, n):
        total = 0
        v = [x, y]
        nscale = 1

        for i in range(n):
            val = self.noise2(v)
            total += val / nscale
            nscale *= nalpha
            v[0] *= nbeta
            v[1] *= nbeta
        return total

    def noise_3d(self, x, y, z, nalpha, nbeta, n):
        total = 0
        v = [x, y, z]
        nscale = 1

        for i in range(n):
            val = self.noise3(v)
            total += val / nscale
            nscale *= nalpha
            v[0] *= nbeta
            v[1] *= nbeta
            v[2] *= nbeta
        return total


run()
