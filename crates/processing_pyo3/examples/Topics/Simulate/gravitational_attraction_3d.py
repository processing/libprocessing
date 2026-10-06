# Gravitational Attraction (3D)
# by Daniel Shiffman.
#
# Simulating gravitational attraction
# G ---> universal gravitational constant
# m1 --> mass of object #1
# m2 --> mass of object #2
# d ---> distance between objects
# F = (G*m1*m2)/(d*d)
#
# For the basics of working with PVector, see
# http://processing.org/learning/pvector/
# as well as examples in Topics/Vectors/
from mewnala import *

# A bunch of planets
planets = []
# One sun (note sun is not attracted to planets (violation of Newton's 3rd Law)
s = None

# An angle to rotate around the scene
angle = 0


def setup():
    global s
    size(640, 360)
    mode_3d()
    # Some random planets
    for i in range(10):
        planets.append(Planet(random(0.1, 2), random(-width / 2, width / 2), random(-height / 2, height / 2), random(-100, 100)))
    # A single sun
    s = Sun()
    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)


def draw():
    global angle
    background(0.0)
    # Setup the scene
    rotate_y(angle)

    # Display the Sun
    s.display()

    # All the Planets
    for planet in planets:
        # Sun attracts Planets
        force = s.attract(planet)
        planet.apply_force(force)
        # Update and draw Planets
        planet.update()
        planet.display()

    # Rotate around the scene
    angle += 0.003


# A class for an orbiting Planet
class Planet:
    def __init__(self, m, x, y, z):
        # Basic physics model (position, velocity, acceleration, mass)
        self.mass = m
        self.position = Vec3(x, y, z)
        self.velocity = Vec3(1, 0, 0)  # Arbitrary starting velocity
        self.acceleration = Vec3(0, 0, 0)

    # Newton's 2nd Law (F = M*A) applied
    def apply_force(self, force):
        f = force / self.mass
        self.acceleration.add(f)

    # Our motion algorithm (aka Euler Integration)
    def update(self):
        self.velocity.add(self.acceleration)  # Velocity changes according to acceleration
        self.position.add(self.velocity)  # position changes according to velocity
        self.acceleration.mult(0)

    # Draw the Planet
    def display(self):
        no_stroke()
        fill(1.0)
        push_matrix()
        translate(self.position.x, self.position.y, self.position.z)
        sphere(self.mass * 8, 8, 8)
        pop_matrix()


# A class for an attractive body in our world
class Sun:
    def __init__(self):
        self.position = Vec3(0, 0, 0)
        self.mass = 20  # Mass, tied to size
        self.G = 0.4  # Universal gravitational constant (arbitrary value)

    def attract(self, m):
        force = self.position - m.position  # Calculate direction of force
        d = force.mag()  # Distance between objects
        d = constrain(d, 5.0, 25.0)  # Limiting the distance to eliminate "extreme" results for very close or very far objects
        strength = (self.G * self.mass * m.mass) / (d * d)  # Calculate gravitional force magnitude
        force.set_mag(strength)  # Get force vector --> magnitude * direction
        return force

    # Draw Sun
    def display(self):
        stroke(1.0)
        no_fill()
        push_matrix()
        translate(self.position.x, self.position.y, self.position.z)
        sphere(self.mass * 2, 8, 8)
        pop_matrix()


run()
