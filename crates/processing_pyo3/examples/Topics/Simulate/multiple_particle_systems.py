# Multiple Particle Systems
# by Daniel Shiffman.
#
# Click the mouse to generate a burst of particles
# at mouse position.
#
# Each burst is one instance of a particle system
# with Particles and CrazyParticles (a subclass of Particle).
# Note use of Inheritance and Polymorphism.
from mewnala import *

systems = []


def setup():
    size(640, 360)


def draw():
    background(0.0)
    for ps in systems:
        ps.run()
        ps.add_particle()
    if len(systems) == 0:
        fill(1.0)
        text_align(CENTER)
        text("click mouse to add particle systems", width / 2, height / 2)


def mouse_pressed():
    systems.append(ParticleSystem(1, Vec2(mouse_x, mouse_y)))


# A simple Particle class
class Particle:
    def __init__(self, l):
        self.acceleration = Vec2(0, 0.05)
        self.velocity = Vec2(random(-1, 1), random(-2, 0))
        self.position = l.copy()
        self.lifespan = 255.0

    def run(self):
        self.update()
        self.display()

    # Method to update position
    def update(self):
        self.velocity.add(self.acceleration)
        self.position.add(self.velocity)
        self.lifespan -= 2.0

    # Method to display
    def display(self):
        stroke(1.0, self.lifespan / 255)
        fill(1.0, self.lifespan / 255)
        ellipse(self.position.x, self.position.y, 8, 8)

    # Is the particle still useful?
    def is_dead(self):
        return self.lifespan < 0.0


# A subclass of Particle
class CrazyParticle(Particle):
    # The CrazyParticle constructor can call the parent class (super class) constructor
    def __init__(self, l):
        # "super" means do everything from the constructor in Particle
        super().__init__(l)
        # One more line of code to deal with the new variable, theta
        # It inherits all other fields from "Particle", and we don't have to retype them!
        self.theta = 0.0

    # Notice we don't have the method run() here; it is inherited from Particle

    # This update() method overrides the parent class update() method
    def update(self):
        super().update()
        # Increment rotation based on horizontal velocity
        theta_vel = (self.velocity.x * self.velocity.mag()) / 10.0
        self.theta += theta_vel

    # This display() method overrides the parent class display() method
    def display(self):
        # Render the ellipse just like in a regular particle
        super().display()
        # Then add a rotating line
        push_matrix()
        translate(self.position.x, self.position.y)
        rotate(self.theta)
        stroke(1.0, self.lifespan / 255)
        line(0, 0, 25, 0)
        pop_matrix()


# A list is used to manage the list of Particles
class ParticleSystem:
    def __init__(self, num, v):
        self.particles = []  # A list for all the particles
        self.origin = v.copy()  # Store the origin point
        for i in range(num):
            self.particles.append(Particle(self.origin))  # Add "num" amount of particles to the list

    def run(self):
        # Cycle through the list backwards, because we are deleting while iterating
        for i in range(len(self.particles) - 1, -1, -1):
            p = self.particles[i]
            p.run()
            if p.is_dead():
                self.particles.pop(i)

    def add_particle(self, p=None):
        if p is None:
            # Add either a Particle or CrazyParticle to the system
            if int(random(0, 2)) == 0:
                p = Particle(self.origin)
            else:
                p = CrazyParticle(self.origin)
        self.particles.append(p)

    # A method to test if the particle system still has particles
    def dead(self):
        return len(self.particles) == 0


run()
