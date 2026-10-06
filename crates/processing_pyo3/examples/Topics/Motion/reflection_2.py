# Non-orthogonal Collision with Multiple Ground Segments
# by Ira Greenberg.
#
# Based on Keith Peter's Solution in
# Foundation Actionscript Animation: Making Things Move!
from mewnala import *

orb = None

gravity = Vec2(0, 0.05)
# The ground is a list of "Ground" objects
segments = 40
ground = []


def setup():
    global orb
    size(640, 360)
    # An orb object that will fall and bounce around
    orb = Orb(50, 50, 3)

    # Calculate ground peak heights
    peak_heights = []
    for i in range(segments + 1):
        peak_heights.append(random(height - 40, height - 30))

    # Float value required for segment width (segs)
    # calculations so the ground spans the entire
    # display window, regardless of segment number.
    segs = float(segments)
    for i in range(segments):
        ground.append(Ground(width / segs * i, peak_heights[i], width / segs * (i + 1), peak_heights[i + 1]))


def draw():
    # Background
    no_stroke()
    fill(0.0, 0.06)
    rect(0, 0, width, height)

    # Move and display the orb
    orb.move()
    orb.display()
    # Check walls
    orb.check_wall_collision()

    # Check against all the ground segments
    for i in range(segments):
        orb.check_ground_collision(ground[i])

    # Draw ground
    fill(0.5)
    begin_shape()
    for i in range(segments):
        vertex(ground[i].x1, ground[i].y1)
        vertex(ground[i].x2, ground[i].y2)
    vertex(ground[segments - 1].x2, height)
    vertex(ground[0].x1, height)
    end_shape(CLOSE)


class Ground:
    def __init__(self, x1, y1, x2, y2):
        self.x1 = x1
        self.y1 = y1
        self.x2 = x2
        self.y2 = y2
        self.x = (x1 + x2) / 2
        self.y = (y1 + y2) / 2
        self.len = dist(x1, y1, x2, y2)
        self.rot = atan2((y2 - y1), (x2 - x1))


class Orb:
    def __init__(self, x, y, r_):
        self.position = Vec2(x, y)
        self.velocity = Vec2(0.5, 0)
        self.r = r_
        # A damping of 80% slows it down when it hits the ground
        self.damping = 0.8

    def move(self):
        # Move orb
        self.velocity.add(gravity)
        self.position.add(self.velocity)

    def display(self):
        # Draw orb
        no_stroke()
        fill(0.78)
        ellipse(self.position.x, self.position.y, self.r * 2, self.r * 2)

    # Check boundaries of window
    def check_wall_collision(self):
        if self.position.x > width - self.r:
            self.position.x = width - self.r
            self.velocity.x *= -self.damping
        elif self.position.x < self.r:
            self.position.x = self.r
            self.velocity.x *= -self.damping

    def check_ground_collision(self, ground_segment):
        # Get difference between orb and ground
        delta_x = self.position.x - ground_segment.x
        delta_y = self.position.y - ground_segment.y

        # Precalculate trig values
        cosine = cos(ground_segment.rot)
        sine = sin(ground_segment.rot)

        # Rotate ground and velocity to allow
        # orthogonal collision calculations
        ground_x_temp = cosine * delta_x + sine * delta_y
        ground_y_temp = cosine * delta_y - sine * delta_x
        velocity_x_temp = cosine * self.velocity.x + sine * self.velocity.y
        velocity_y_temp = cosine * self.velocity.y - sine * self.velocity.x

        # Ground collision - check for surface
        # collision and also that orb is within
        # left/rights bounds of ground segment
        if ground_y_temp > -self.r and self.position.x > ground_segment.x1 and self.position.x < ground_segment.x2:
            # keep orb from going into ground
            ground_y_temp = -self.r
            # bounce and slow down orb
            velocity_y_temp *= -1.0
            velocity_y_temp *= self.damping

        # Reset ground, velocity and orb
        delta_x = cosine * ground_x_temp - sine * ground_y_temp
        delta_y = cosine * ground_y_temp + sine * ground_x_temp
        self.velocity.x = cosine * velocity_x_temp - sine * velocity_y_temp
        self.velocity.y = cosine * velocity_y_temp + sine * velocity_x_temp
        self.position.x = ground_segment.x + delta_x
        self.position.y = ground_segment.y + delta_y


run()
