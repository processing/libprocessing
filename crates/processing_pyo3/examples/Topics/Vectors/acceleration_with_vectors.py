# Acceleration with Vectors
# by Daniel Shiffman.
#
# Demonstration of the basics of motion with vector.
# A "Mover" object stores location, velocity, and
# acceleration as vectors. The motion is controlled by
# affecting the acceleration (in this case towards the mouse).
from mewnala import *

# A Mover object
mover = None


def setup():
    global mover
    size(640, 360)
    mover = Mover()


def draw():
    background(0.0)

    # Update the location
    mover.update()
    # Display the Mover
    mover.display()


class Mover:
    def __init__(self):
        # Start in the center
        self.location = Vec2(width / 2, height / 2)
        self.velocity = Vec2(0, 0)
        # The Mover's maximum speed
        self.topspeed = 5

    def update(self):
        # Compute a vector that points from location to mouse
        mouse = Vec2(mouse_x, mouse_y)
        acceleration = mouse - self.location
        # Set magnitude of acceleration
        acceleration.set_mag(0.2)

        # Velocity changes according to acceleration
        self.velocity.add(acceleration)
        # Limit the velocity by topspeed
        self.velocity.limit(self.topspeed)
        # Location changes by velocity
        self.location.add(self.velocity)

    def display(self):
        stroke(1.0)
        stroke_weight(2)
        fill(0.5)
        ellipse(self.location.x, self.location.y, 48, 48)


run()
