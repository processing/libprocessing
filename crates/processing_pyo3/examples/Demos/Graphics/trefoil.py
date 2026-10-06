# Trefoil, by Andres Colubri
#
# A parametric surface is textured procedurally
# by drawing on an offscreen Graphics surface.
from mewnala import *

pg = None
trefoil = None


def setup():
    global pg, trefoil
    size(1024, 768)
    mode_3d()

    no_stroke()

    # Creating offscreen surface for 3D rendering.
    pg = create_graphics(32, 512)
    pg.begin_draw()
    pg.background(0.0, 0.0)
    pg.no_stroke()
    pg.fill(1.0, 0.0, 0.0, 0.78)
    pg.end_draw()

    # Saving trefoil surface into a geometry
    trefoil = create_trefoil(350, 60, 15)

    # ambient(250, 250, 250); pointLight(255, 255, 255, 0, 0, 200)
    point_light((1.0, 1.0, 1.0), 1000000.0, 600.0, 1.0, position=(0, 0, 200))
    roughness(0.6)


def draw():
    background(0.0)

    pg.begin_draw()
    pg.ellipse(random(pg.width), random(pg.height), 4, 4)
    pg.end_draw()

    push_matrix()
    translate(0, 0, -200)
    rotate_x(frame_count * PI / 500)
    rotate_y(frame_count * PI / 500)
    texture(pg)
    draw_geometry(trefoil)
    no_texture()
    pop_matrix()


# Code to draw a trefoil knot surface, with normals and texture
# coordinates.
# Adapted from the parametric equations example by Philip Rideout:
# http://iphone-3d-programming.labs.oreilly.com/ch03.html

# This function builds a trefoil knot surface as a triangle mesh derived
# from its parametric equation.
def create_trefoil(s, ny, nx):
    obj = create_geometry(topology=TRIANGLES)

    for j in range(nx):
        u0 = j / nx
        u1 = (j + 1) / nx
        for i in range(ny):
            v0 = i / ny
            v1 = (i + 1) / ny

            p0 = eval_point(u0, v0)
            n0 = eval_normal(u0, v0)

            p1 = eval_point(u0, v1)
            n1 = eval_normal(u0, v1)

            p2 = eval_point(u1, v1)
            n2 = eval_normal(u1, v1)

            # Triangle p0-p1-p2
            obj.normal(n0.x, n0.y, n0.z)
            obj.uv(u0, v0)
            obj.vertex(s * p0.x, s * p0.y, s * p0.z)
            obj.normal(n1.x, n1.y, n1.z)
            obj.uv(u0, v1)
            obj.vertex(s * p1.x, s * p1.y, s * p1.z)
            obj.normal(n2.x, n2.y, n2.z)
            obj.uv(u1, v1)
            obj.vertex(s * p2.x, s * p2.y, s * p2.z)

            p1 = eval_point(u1, v0)
            n1 = eval_normal(u1, v0)

            # Triangle p0-p2-p1
            obj.normal(n0.x, n0.y, n0.z)
            obj.uv(u0, v0)
            obj.vertex(s * p0.x, s * p0.y, s * p0.z)
            obj.normal(n2.x, n2.y, n2.z)
            obj.uv(u1, v1)
            obj.vertex(s * p2.x, s * p2.y, s * p2.z)
            obj.normal(n1.x, n1.y, n1.z)
            obj.uv(u1, v0)
            obj.vertex(s * p1.x, s * p1.y, s * p1.z)
    return obj


# Evaluates the surface normal corresponding to normalized
# parameters (u, v)
def eval_normal(u, v):
    # Compute the tangents and their cross product.
    p = eval_point(u, v)
    tang_u = eval_point(u + 0.01, v)
    tang_v = eval_point(u, v + 0.01)
    tang_u.sub(p)
    tang_v.sub(p)

    norm_uv = tang_v.cross(tang_u)
    norm_uv.normalize()
    return norm_uv


# Evaluates the surface point corresponding to normalized
# parameters (u, v)
def eval_point(u, v):
    a = 0.5
    b = 0.3
    c = 0.5
    d = 0.1
    s = TWO_PI * u
    t = (TWO_PI * (1 - v)) * 2

    r = a + b * cos(1.5 * t)
    x = r * cos(t)
    y = r * sin(t)
    z = c * sin(1.5 * t)

    dv = Vec3()
    dv.x = -1.5 * b * sin(1.5 * t) * cos(t) - (a + b * cos(1.5 * t)) * sin(t)
    dv.y = -1.5 * b * sin(1.5 * t) * sin(t) + (a + b * cos(1.5 * t)) * cos(t)
    dv.z = 1.5 * c * cos(1.5 * t)

    q = dv
    q.normalize()
    qvn = Vec3(q.y, -q.x, 0)
    qvn.normalize()
    ww = q.cross(qvn)

    pt = Vec3()
    pt.x = x + d * (qvn.x * cos(s) + ww.x * sin(s))
    pt.y = y + d * (qvn.y * cos(s) + ww.y * sin(s))
    pt.z = z + d * ww.z * sin(s)
    return pt


run()
