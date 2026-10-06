# Simple Particle System
# by Daniel Shiffman.
#
# Particles are generated each cycle through draw(),
# fall with gravity, and fade out over time.
# A ParticleSystem object manages a variable size
# list of particles.
from mewnala import *

ps = None


def setup():
    global ps
    size(640, 360)
    ps = ParticleSystem(Vec2(width / 2, 50))


def draw():
    background(0.0)
    ps.add_particle()
    ps.run()


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
        self.lifespan -= 1.0

    # Method to display
    def display(self):
        stroke(1.0, self.lifespan / 255)
        fill(1.0, self.lifespan / 255)
        ellipse(self.position.x, self.position.y, 8, 8)

    # Is the particle still useful?
    def is_dead(self):
        if self.lifespan < 0.0:
            return True
        else:
            return False


# A class to describe a group of Particles
# A list is used to manage the list of Particles
class ParticleSystem:
    def __init__(self, position):
        self.origin = position.copy()
        self.particles = []

    def add_particle(self):
        self.particles.append(Particle(self.origin))

    def run(self):
        for i in range(len(self.particles) - 1, -1, -1):
            p = self.particles[i]
            p.run()
            if p.is_dead():
                self.particles.pop(i)


run()
