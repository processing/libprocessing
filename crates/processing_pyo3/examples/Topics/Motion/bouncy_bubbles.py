# Bouncy Bubbles
# based on code from Keith Peters.
#
# Multiple-object collision.
from mewnala import *

num_balls = 12
spring = 0.05
gravity = 0.03
friction = -0.9
balls = []


def setup():
    size(640, 360)
    for i in range(num_balls):
        balls.append(Ball(random(width), random(height), random(30, 70), i, balls))
    no_stroke()
    fill(1.0, 0.8)


def draw():
    background(0.0)
    for ball in balls:
        ball.collide()
        ball.move()
        ball.display()


class Ball:
    def __init__(self, xin, yin, din, idin, oin):
        self.x = xin
        self.y = yin
        self.diameter = din
        self.vx = 0
        self.vy = 0
        self.id = idin
        self.others = oin

    def collide(self):
        for i in range(self.id + 1, num_balls):
            dx = self.others[i].x - self.x
            dy = self.others[i].y - self.y
            distance = sqrt(dx * dx + dy * dy)
            min_dist = self.others[i].diameter / 2 + self.diameter / 2
            if distance < min_dist:
                angle = atan2(dy, dx)
                target_x = self.x + cos(angle) * min_dist
                target_y = self.y + sin(angle) * min_dist
                ax = (target_x - self.others[i].x) * spring
                ay = (target_y - self.others[i].y) * spring
                self.vx -= ax
                self.vy -= ay
                self.others[i].vx += ax
                self.others[i].vy += ay

    def move(self):
        self.vy += gravity
        self.x += self.vx
        self.y += self.vy
        if self.x + self.diameter / 2 > width:
            self.x = width - self.diameter / 2
            self.vx *= friction
        elif self.x - self.diameter / 2 < 0:
            self.x = self.diameter / 2
            self.vx *= friction
        if self.y + self.diameter / 2 > height:
            self.y = height - self.diameter / 2
            self.vy *= friction
        elif self.y - self.diameter / 2 < 0:
            self.y = self.diameter / 2
            self.vy *= friction

    def display(self):
        ellipse(self.x, self.y, self.diameter, self.diameter)


run()
