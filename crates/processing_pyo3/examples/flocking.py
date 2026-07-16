# Flocking
#
# Ported from Dan Schiffman's Flocking
#
# An implementation of Craig Reynold's Boids program to simulate
# the flocking behavior of birds. Each boid steers itself based on
# rules of avoidance, alignment, and coherence.
#
# Click the mouse to add a new boid.
from mewnala import *

flock = None

title_last_time = 0.0
title_last_frame = 0
boid_count = 150

def setup():
    global flock
    size(640, 360)
    flock = Flock()
    # Add an initial set of boids into the system
    for i in range(boid_count):
        flock.add_boid(Boid(width / 2, height / 2))


def draw():
    global title_last_time, title_last_frame

    title_elapsed = elapsed_time - title_last_time
    if title_elapsed >= 0.5:
        fps = (frame_count - title_last_frame) / title_elapsed
        window_title(f"GPU Flocking Duck — {boid_count:,} boids — {fps:.0f} FPS")
        title_last_time = elapsed_time
        title_last_frame = frame_count

    background(50)
    flock.run()


# Add a new boid into the System
def mouse_pressed():
    flock.add_boid(Boid(mouse_x, mouse_y))


# The Flock (a list of Boid objects)
class Flock:
    def __init__(self):
        self.boids = []  # A list for all the boids

    def run(self):
        for b in self.boids:
            b.run(self.boids)  # Passing the entire list of boids to each boid individually

    def add_boid(self, b):
        self.boids.append(b)


# The Boid class
class Boid:
    def __init__(self, x, y):
        self.acceleration = Vec2(0, 0)
        self.velocity = Vec2.random()
        self.position = Vec2(x, y)
        self.r = 2.0
        self.maxspeed = 2.0  # Maximum speed
        self.maxforce = 0.03  # Maximum steering force

    def run(self, boids):
        self.flock(boids)
        self.update()
        self.borders()
        self.render()

    def apply_force(self, force):
        # We could add mass here if we want A = F / M
        self.acceleration.add(force)

    # We accumulate a new acceleration each time based on three rules
    def flock(self, boids):
        sep = self.separate(boids)  # Separation
        ali = self.align(boids)     # Alignment
        coh = self.cohesion(boids)  # Cohesion
        # Arbitrarily weight these forces
        sep.mult(1.5)
        ali.mult(1.0)
        coh.mult(1.0)
        # Add the force vectors to acceleration
        self.apply_force(sep)
        self.apply_force(ali)
        self.apply_force(coh)

    # Method to update position
    def update(self):
        # Update velocity
        self.velocity.add(self.acceleration)
        # Limit speed
        self.velocity.limit(self.maxspeed)
        self.position.add(self.velocity)
        # Reset acceleration to 0 each cycle
        self.acceleration.mult(0)

    # A method that calculates and applies a steering force towards a target
    # STEER = DESIRED MINUS VELOCITY
    def seek(self, target):
        desired = target - self.position  # A vector pointing from the position to the target
        # Scale to maximum speed
        desired.set_mag(self.maxspeed)

        # Steering = Desired minus Velocity
        steer = desired - self.velocity
        steer.limit(self.maxforce)  # Limit to maximum steering force
        return steer

    def render(self):
        # Draw a triangle rotated in the direction of velocity
        theta = self.velocity.heading() + HALF_PI

        fill(200, 100)
        stroke(255)
        push_matrix()
        translate(self.position.x, self.position.y)
        rotate(theta)
        begin_shape(TRIANGLES)
        vertex(0, -self.r * 2)
        vertex(-self.r, self.r * 2)
        vertex(self.r, self.r * 2)
        end_shape()
        pop_matrix()

    # Wraparound
    def borders(self):
        if self.position.x < -self.r:
            self.position.x = width + self.r
        if self.position.y < -self.r:
            self.position.y = height + self.r
        if self.position.x > width + self.r:
            self.position.x = -self.r
        if self.position.y > height + self.r:
            self.position.y = -self.r

    # Separation
    # Method checks for nearby boids and steers away
    def separate(self, boids):
        desired_separation = 25.0
        steer = Vec2(0, 0)
        count = 0
        # For every boid in the system, check if it's too close
        for other in boids:
            d = self.position.dist(other.position)
            # If the distance is greater than 0 and less than an arbitrary amount (0 when you are yourself)
            if 0 < d < desired_separation:
                # Calculate vector pointing away from neighbor
                diff = (self.position - other.position).normalize()
                diff.div(d)  # Weight by distance
                steer.add(diff)
                count += 1  # Keep track of how many
        # Average -- divide by how many
        if count > 0:
            steer.div(count)

        # As long as the vector is greater than 0
        if steer.mag() > 0:
            # Implement Reynolds: Steering = Desired - Velocity
            steer.set_mag(self.maxspeed)
            steer.sub(self.velocity)
            steer.limit(self.maxforce)
        return steer

    # Alignment
    # For every nearby boid in the system, calculate the average velocity
    def align(self, boids):
        neighbor_dist = 50.0
        sum = Vec2(0, 0)
        count = 0
        for other in boids:
            d = self.position.dist(other.position)
            if 0 < d < neighbor_dist:
                sum.add(other.velocity)
                count += 1
        if count > 0:
            sum.div(count)
            # Implement Reynolds: Steering = Desired - Velocity
            sum.set_mag(self.maxspeed)
            steer = sum - self.velocity
            steer.limit(self.maxforce)
            return steer
        else:
            return Vec2(0, 0)

    # Cohesion
    # For the average position (i.e. center) of all nearby boids, calculate steering vector towards that position
    def cohesion(self, boids):
        neighbor_dist = 50.0
        sum = Vec2(0, 0)  # Start with empty vector to accumulate all positions
        count = 0
        for other in boids:
            d = self.position.dist(other.position)
            if 0 < d < neighbor_dist:
                sum.add(other.position)  # Add position
                count += 1
        if count > 0:
            sum.div(count)
            return self.seek(sum)  # Steer towards the position
        else:
            return Vec2(0, 0)


run()
