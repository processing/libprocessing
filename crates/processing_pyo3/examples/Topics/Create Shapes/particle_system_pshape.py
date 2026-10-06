# ParticleSystemPShape
#
# A particle system optimized for drawing using PShape
from mewnala import *

# Particle System object
ps = None
# An image for the particle's texture
sprite = None


def setup():
    global ps, sprite
    size(640, 360)
    # Load the image
    sprite = load_image("data/sprite.png")
    # A new particle system with 10,000 particles
    ps = ParticleSystem(10000)


def draw():
    background(0.0)
    # Update and display system
    ps.update()
    ps.display()

    # Set the particle system's emitter location to the mouse
    ps.set_emitter(mouse_x, mouse_y)

    # Display frame rate
    fill(1.0)
    text_size(16)
    text("Frame rate: " + str(int(frame_rate())), 10, 20)


# The Particle System
class ParticleSystem:
    def __init__(self, n):
        # It's just a list of particle objects
        self.particles = []
        # The shape to group all the particle shapes
        self.particle_shape = create_shape(GROUP)  # GAP: create_shape(GROUP) retained shape group

        # Make all the Particles
        for i in range(n):
            p = Particle()
            self.particles.append(p)
            # Each particle's shape gets added to the System shape
            self.particle_shape.add_child(p.get_shape())  # GAP: Shape.add_child

    def update(self):
        for p in self.particles:
            p.update()

    def set_emitter(self, x, y):
        for p in self.particles:
            # Each particle gets reborn at the emitter location
            if p.is_dead():
                p.rebirth(x, y)

    def display(self):
        shape(self.particle_shape)  # GAP: shape(s) draws a retained shape (here a group of 10,000 textured quads)


# An individual Particle
class Particle:
    def __init__(self):
        # The particle size
        self.part_size = random(10, 60)
        # The particle is a textured quad
        self.part = create_shape()  # GAP: create_shape() retained shape with its own style
        self.part.begin_shape(QUADS)  # GAP: Shape.begin_shape(kind)
        self.part.no_stroke()  # GAP: Shape.no_stroke
        self.part.texture(sprite)  # GAP: Shape.texture(img)
        self.part.normal(0, 0, 1)  # GAP: Shape.normal
        self.part.vertex(-self.part_size / 2, -self.part_size / 2, 0, 0)  # GAP: Shape.vertex(x, y, u, v)
        self.part.vertex(self.part_size / 2, -self.part_size / 2, 1, 0)
        self.part.vertex(self.part_size / 2, self.part_size / 2, 1, 1)
        self.part.vertex(-self.part_size / 2, self.part_size / 2, 0, 1)
        self.part.end_shape()  # GAP: Shape.end_shape

        # Lifespan is tied to alpha
        self.lifespan = 255
        # A single force
        self.gravity = Vec2(0, 0.1)
        # Velocity
        self.velocity = Vec2(0, 0)
        # Initialize center vector
        self.center = Vec2(0, 0)

        # Set the particle starting location
        self.rebirth(width / 2, height / 2)

    def get_shape(self):
        return self.part

    def rebirth(self, x, y):
        a = random(TWO_PI)
        speed = random(0.5, 4)
        # A velocity with random angle and magnitude
        self.velocity = Vec2.from_angle(a)
        self.velocity.mult(speed)
        # Set lifespan
        self.lifespan = 255
        # Set location using translate
        self.part.reset_matrix()  # GAP: Shape.reset_matrix (shape-local transform)
        self.part.translate(x, y)  # GAP: Shape.translate (shape-local transform)

        # Update center vector
        self.center.set(x, y)

    # Is it off the screen, or its lifespan is over?
    def is_dead(self):
        if self.center.x > width or self.center.x < 0 or self.center.y > height or self.center.y < 0 or self.lifespan < 0:
            return True
        else:
            return False

    def update(self):
        # Decrease life
        self.lifespan = self.lifespan - 1
        # Apply gravity
        self.velocity.add(self.gravity)
        self.part.set_tint(color(1.0, self.lifespan / 255))  # GAP: Shape.set_tint(color)
        # Move the particle according to its velocity
        self.part.translate(self.velocity.x, self.velocity.y)
        # and also update the center
        self.center.add(self.velocity)


run()
