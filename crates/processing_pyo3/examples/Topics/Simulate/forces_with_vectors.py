# Forces (Gravity and Fluid Resistence) with Vectors
# by Daniel Shiffman.
#
# Demonstration of multiple forces acting on bodies.
# Bodies experience gravity continuously and fluid
# resistance when in "water".
from mewnala import *

# Ten moving bodies
movers = []

# Liquid
liquid = None


def setup():
    global liquid
    size(640, 360)
    reset()
    # Create liquid object
    liquid = Liquid(0, height / 2, width, height / 2, 0.1)


def draw():
    background(0.0)

    # Draw water
    liquid.display()

    for mover in movers:
        # Is the Mover in the liquid?
        if liquid.contains(mover):
            # Calculate drag force
            drag = liquid.drag(mover)
            # Apply drag force to Mover
            mover.apply_force(drag)

        # Gravity is scaled by mass here!
        gravity = Vec2(0, 0.1 * mover.mass)
        # Apply gravity
        mover.apply_force(gravity)

        # Update and display
        mover.update()
        mover.display()
        mover.check_edges()

    fill(1.0)
    text("click mouse to reset", 10, 30)


def mouse_pressed():
    reset()


# Restart all the Mover objects randomly
def reset():
    movers.clear()
    for i in range(10):
        movers.append(Mover(random(0.5, 3), 40 + i * 70, 0))


class Liquid:
    def __init__(self, x_, y_, w_, h_, c_):
        # Liquid is a rectangle
        self.x = x_
        self.y = y_
        self.w = w_
        self.h = h_
        # Coefficient of drag
        self.c = c_

    # Is the Mover in the Liquid?
    def contains(self, m):
        l = m.position
        if l.x > self.x and l.x < self.x + self.w and l.y > self.y and l.y < self.y + self.h:
            return True
        else:
            return False

    # Calculate drag force
    def drag(self, m):
        # Magnitude is coefficient * speed squared
        speed = m.velocity.mag()
        drag_magnitude = self.c * speed * speed

        # Direction is inverse of velocity
        drag = m.velocity.copy()
        drag.mult(-1)

        # Scale according to magnitude
        drag.set_mag(drag_magnitude)
        return drag

    def display(self):
        no_stroke()
        fill(0.5)
        rect(self.x, self.y, self.w, self.h)


class Mover:
    def __init__(self, m, x, y):
        # Mass is tied to size
        self.mass = m
        self.position = Vec2(x, y)
        self.velocity = Vec2(0, 0)
        self.acceleration = Vec2(0, 0)

    # Newton's 2nd law: F = M * A
    # or A = F / M
    def apply_force(self, force):
        # Divide by mass
        f = force / self.mass
        # Accumulate all forces in acceleration
        self.acceleration.add(f)

    def update(self):
        # Velocity changes according to acceleration
        self.velocity.add(self.acceleration)
        # position changes by velocity
        self.position.add(self.velocity)
        # We must clear acceleration each frame
        self.acceleration.mult(0)

    # Draw Mover
    def display(self):
        stroke(1.0)
        stroke_weight(2)
        fill(1.0, 0.78)
        ellipse(self.position.x, self.position.y, self.mass * 16, self.mass * 16)

    # Bounce off bottom of window
    def check_edges(self):
        if self.position.y > height:
            self.velocity.y *= -0.9  # A little dampening when hitting the bottom
            self.position.y = height


run()
