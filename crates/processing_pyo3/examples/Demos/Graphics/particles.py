# Particles, by Daniel Shiffman.
#
# Each particle is a textured quad (the sprite image) that fades
# out over its lifespan; dead particles are reborn at the mouse.
# Ported 1:1 on the CPU: the quads are drawn with image() and tint().
from mewnala import *

ps = None
sprite = None


def setup():
    global ps, sprite
    size(1024, 768)
    sprite = load_image("data/sprite.png")
    ps = ParticleSystem(10000)


def draw():
    background(0.0)
    ps.update()
    ps.display()

    ps.set_emitter(mouse_x, mouse_y)

    no_tint()
    fill(1.0)
    text_size(16)
    text("Frame rate: " + str(int(frame_rate())), 10, 20)


class Particle:
    def __init__(self):
        self.velocity = None
        self.lifespan = 255
        self.part_size = random(10, 60)
        self.gravity = Vec2(0, 0.1)
        self.x = 0
        self.y = 0

        self.rebirth(width / 2, height / 2)
        self.lifespan = random(255)

    def rebirth(self, x, y):
        a = random(TWO_PI)
        speed = random(0.5, 4)
        self.velocity = Vec2(cos(a), sin(a))
        self.velocity.mult(speed)
        self.lifespan = 255
        self.x = x
        self.y = y

    def is_dead(self):
        if self.lifespan < 0:
            return True
        else:
            return False

    def update(self):
        self.lifespan = self.lifespan - 1
        self.velocity.add(self.gravity)

        self.x += self.velocity.x
        self.y += self.velocity.y

    def display(self):
        tint(1.0, self.lifespan / 255)
        image(sprite, self.x - self.part_size / 2, self.y - self.part_size / 2, self.part_size, self.part_size)


class ParticleSystem:
    def __init__(self, n):
        self.particles = []
        for i in range(n):
            self.particles.append(Particle())

    def update(self):
        for p in self.particles:
            p.update()

    def set_emitter(self, x, y):
        for p in self.particles:
            if p.is_dead():
                p.rebirth(x, y)

    def display(self):
        no_stroke()
        for p in self.particles:
            p.display()


run()
