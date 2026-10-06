# Smoke Particle System
# by Daniel Shiffman.
#
# A basic smoke effect using a particle system. Each particle
# is rendered as an alpha masked image.
from mewnala import *

ps = None


def setup():
    global ps
    size(640, 360)
    img = load_image("data/texture.png")
    ps = ParticleSystem(0, Vec2(width / 2, height - 60), img)


def draw():
    background(0.0)

    # Calculate a "wind" force based on mouse horizontal position
    dx = remap(mouse_x, 0, width, -0.2, 0.2)
    wind = Vec2(dx, 0)
    ps.apply_force(wind)
    ps.run()
    for i in range(2):
        ps.add_particle()

    # Draw an arrow representing the wind force
    draw_vector(wind, Vec2(width / 2, 50), 500)


# Renders a vector object 'v' as an arrow and a position 'loc'
def draw_vector(v, loc, scayl):
    push_matrix()
    arrowsize = 4
    # Translate to position to render vector
    translate(loc.x, loc.y)
    stroke(1.0)
    # Call vector heading function to get direction (note that pointing up is a heading of 0) and rotate
    rotate(v.heading())
    # Calculate length of vector & scale it to be bigger or smaller if necessary
    len = v.mag() * scayl
    # Draw three lines to make an arrow (draw pointing up since we've rotate to the proper direction)
    line(0, 0, len, 0)
    line(len, 0, len - arrowsize, +arrowsize / 2)
    line(len, 0, len - arrowsize, -arrowsize / 2)
    pop_matrix()


# A simple Particle class, renders the particle as an image
class Particle:
    def __init__(self, l, img_):
        self.acc = Vec2(0, 0)
        vx = random_gaussian() * 0.3
        vy = random_gaussian() * 0.3 - 1.0
        self.vel = Vec2(vx, vy)
        self.loc = l.copy()
        self.lifespan = 100.0
        self.img = img_

    def run(self):
        self.update()
        self.render()

    # Method to apply a force vector to the Particle object
    # Note we are ignoring "mass" here
    def apply_force(self, f):
        self.acc.add(f)

    # Method to update position
    def update(self):
        self.vel.add(self.acc)
        self.loc.add(self.vel)
        self.lifespan -= 2.5
        self.acc.mult(0)  # clear Acceleration

    # Method to display
    def render(self):
        image_mode(CENTER)
        tint(1.0, self.lifespan / 255)
        image(self.img, self.loc.x, self.loc.y)

    # Is the particle still useful?
    def is_dead(self):
        if self.lifespan <= 0.0:
            return True
        else:
            return False


# A class to describe a group of Particles
# A list is used to manage the list of Particles
class ParticleSystem:
    def __init__(self, num, v, img_):
        self.particles = []  # A list for all the particles
        self.origin = v.copy()  # An origin point for where particles are birthed
        self.img = img_
        for i in range(num):
            self.particles.append(Particle(self.origin, self.img))  # Add "num" amount of particles to the list

    def run(self):
        for i in range(len(self.particles) - 1, -1, -1):
            p = self.particles[i]
            p.run()
            if p.is_dead():
                self.particles.pop(i)

    # Method to add a force vector to all particles currently in the system
    def apply_force(self, dir):
        for p in self.particles:
            p.apply_force(dir)

    def add_particle(self):
        self.particles.append(Particle(self.origin, self.img))


run()
