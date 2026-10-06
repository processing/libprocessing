# Cubes Contained Within a Cube
# by Ira Greenberg.
#
# Collision detection against all
# outer cube's surfaces.
from mewnala import *

# 20 little internal cubes
cubies = []

# Size of outer cube
bounds = 300


def setup():
    size(640, 360)
    mode_3d()

    for i in range(20):
        # Cubies are randomly sized
        cubie_size = random(5, 15)
        cubies.append(Cube(cubie_size, cubie_size, cubie_size))

    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)


def draw():
    background(0.2)

    # Center in display window
    translate(0, 0, -130)

    # Rotate everything, including external large cube
    rotate_x(frame_count * 0.001)
    rotate_y(frame_count * 0.002)
    rotate_z(frame_count * 0.001)
    stroke(1.0)

    # Outer transparent cube, just using box() method
    no_fill()
    box(bounds)

    # Move and rotate cubies
    for c in cubies:
        c.update()
        c.display()


# Custom Cube Class
class Cube:
    def __init__(self, w, h, d):
        self.w = w
        self.h = h
        self.d = d

        # Colors are hardcoded
        self.quad_bg = [color(0.0), color(0.2), color(0.4), color(0.6), color(0.8), color(1.0)]

        # Start in center
        self.position = Vec3()
        # Random velocity vector
        self.velocity = Vec3.random()
        # Random rotation
        self.rotation = Vec3(random(40, 100), random(40, 100), random(40, 100))

        # cube composed of 6 quads
        self.vertices = [
            # front
            Vec3(-w / 2, -h / 2, d / 2),
            Vec3(w / 2, -h / 2, d / 2),
            Vec3(w / 2, h / 2, d / 2),
            Vec3(-w / 2, h / 2, d / 2),
            # left
            Vec3(-w / 2, -h / 2, d / 2),
            Vec3(-w / 2, -h / 2, -d / 2),
            Vec3(-w / 2, h / 2, -d / 2),
            Vec3(-w / 2, h / 2, d / 2),
            # right
            Vec3(w / 2, -h / 2, d / 2),
            Vec3(w / 2, -h / 2, -d / 2),
            Vec3(w / 2, h / 2, -d / 2),
            Vec3(w / 2, h / 2, d / 2),
            # back
            Vec3(-w / 2, -h / 2, -d / 2),
            Vec3(w / 2, -h / 2, -d / 2),
            Vec3(w / 2, h / 2, -d / 2),
            Vec3(-w / 2, h / 2, -d / 2),
            # top
            Vec3(-w / 2, -h / 2, d / 2),
            Vec3(-w / 2, -h / 2, -d / 2),
            Vec3(w / 2, -h / 2, -d / 2),
            Vec3(w / 2, -h / 2, d / 2),
            # bottom
            Vec3(-w / 2, h / 2, d / 2),
            Vec3(-w / 2, h / 2, -d / 2),
            Vec3(w / 2, h / 2, -d / 2),
            Vec3(w / 2, h / 2, d / 2),
        ]

        # 3D vertices are not immediate-mode: build each quad once as two triangles
        self.faces = []
        for i in range(6):
            a = self.vertices[4 * i]
            b = self.vertices[4 * i + 1]
            c = self.vertices[4 * i + 2]
            n = (b - a).cross(c - a).normalize()
            g = create_geometry(topology=TRIANGLES)
            for j in range(4):
                v = self.vertices[j + 4 * i]
                g.vertex(v.x, v.y, v.z)
                g.normal(n.x, n.y, n.z)
            for k in (0, 1, 2, 0, 2, 3):
                g.index(k)
            self.faces.append(g)

    # Cube shape itself
    def draw_cube(self):
        for i in range(6):
            fill(self.quad_bg[i])
            draw_geometry(self.faces[i])

    # Update location
    def update(self):
        self.position.add(self.velocity)

        # Check wall collisions
        if self.position.x > bounds / 2 or self.position.x < -bounds / 2:
            self.velocity.x *= -1
        if self.position.y > bounds / 2 or self.position.y < -bounds / 2:
            self.velocity.y *= -1
        if self.position.z > bounds / 2 or self.position.z < -bounds / 2:
            self.velocity.z *= -1

    # Display method
    def display(self):
        push_matrix()
        translate(self.position.x, self.position.y, self.position.z)
        rotate_x(frame_count * PI / self.rotation.x)
        rotate_y(frame_count * PI / self.rotation.y)
        rotate_z(frame_count * PI / self.rotation.z)
        no_stroke()
        self.draw_cube()
        pop_matrix()


run()
