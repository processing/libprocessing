# Circle Collision with Swapping Velocities
# by Ira Greenberg.
#
# Based on Keith Peter's Solution in
# Foundation Actionscript Animation: Making Things Move!
from mewnala import *

balls = []


def setup():
    size(640, 360)
    balls.append(Ball(100, 400, 20))
    balls.append(Ball(700, 400, 80))


def draw():
    background(0.2)

    for b in balls:
        b.update()
        b.display()
        b.check_boundary_collision()

    balls[0].check_collision(balls[1])


class Ball:
    def __init__(self, x, y, r_):
        self.position = Vec2(x, y)
        self.velocity = Vec2.random()
        self.velocity.mult(3)
        self.radius = r_
        self.m = self.radius * 0.1

    def update(self):
        self.position.add(self.velocity)

    def check_boundary_collision(self):
        if self.position.x > width - self.radius:
            self.position.x = width - self.radius
            self.velocity.x *= -1
        elif self.position.x < self.radius:
            self.position.x = self.radius
            self.velocity.x *= -1
        elif self.position.y > height - self.radius:
            self.position.y = height - self.radius
            self.velocity.y *= -1
        elif self.position.y < self.radius:
            self.position.y = self.radius
            self.velocity.y *= -1

    def check_collision(self, other):
        # Get distances between the balls components
        distance_vect = other.position - self.position

        # Calculate magnitude of the vector separating the balls
        distance_vect_mag = distance_vect.mag()

        # Minimum distance before they are touching
        min_distance = self.radius + other.radius

        if distance_vect_mag < min_distance:
            distance_correction = (min_distance - distance_vect_mag) / 2.0
            d = distance_vect.copy()
            correction_vector = d.normalize()
            correction_vector.mult(distance_correction)
            other.position.add(correction_vector)
            self.position.sub(correction_vector)

            # get angle of distance_vect
            theta = distance_vect.heading()
            # precalculate trig values
            sine = sin(theta)
            cosine = cos(theta)

            # b_temp will hold rotated ball positions. You
            # just need to worry about b_temp[1] position
            b_temp = [Vec2(), Vec2()]

            # this ball's position is relative to the other
            # so you can use the vector between them (b_vect) as the
            # reference point in the rotation expressions.
            # b_temp[0] stays at 0.0, which is what you want
            # since b[1] will rotate around b[0]
            b_temp[1].x = cosine * distance_vect.x + sine * distance_vect.y
            b_temp[1].y = cosine * distance_vect.y - sine * distance_vect.x

            # rotate Temporary velocities
            v_temp = [Vec2(), Vec2()]

            v_temp[0].x = cosine * self.velocity.x + sine * self.velocity.y
            v_temp[0].y = cosine * self.velocity.y - sine * self.velocity.x
            v_temp[1].x = cosine * other.velocity.x + sine * other.velocity.y
            v_temp[1].y = cosine * other.velocity.y - sine * other.velocity.x

            # Now that velocities are rotated, you can use 1D
            # conservation of momentum equations to calculate
            # the final velocity along the x-axis.
            v_final = [Vec2(), Vec2()]

            # final rotated velocity for b[0]
            v_final[0].x = ((self.m - other.m) * v_temp[0].x + 2 * other.m * v_temp[1].x) / (self.m + other.m)
            v_final[0].y = v_temp[0].y

            # final rotated velocity for b[1]
            v_final[1].x = ((other.m - self.m) * v_temp[1].x + 2 * self.m * v_temp[0].x) / (self.m + other.m)
            v_final[1].y = v_temp[1].y

            # hack to avoid clumping
            b_temp[0].x += v_final[0].x
            b_temp[1].x += v_final[1].x

            # Rotate ball positions and velocities back
            # Reverse signs in trig expressions to rotate
            # in the opposite direction
            b_final = [Vec2(), Vec2()]

            b_final[0].x = cosine * b_temp[0].x - sine * b_temp[0].y
            b_final[0].y = cosine * b_temp[0].y + sine * b_temp[0].x
            b_final[1].x = cosine * b_temp[1].x - sine * b_temp[1].y
            b_final[1].y = cosine * b_temp[1].y + sine * b_temp[1].x

            # update balls to screen position
            other.position.x = self.position.x + b_final[1].x
            other.position.y = self.position.y + b_final[1].y

            self.position.add(b_final[0])

            # update velocities
            self.velocity.x = cosine * v_final[0].x - sine * v_final[0].y
            self.velocity.y = cosine * v_final[0].y + sine * v_final[0].x
            other.velocity.x = cosine * v_final[1].x - sine * v_final[1].y
            other.velocity.y = cosine * v_final[1].y + sine * v_final[1].x

    def display(self):
        no_stroke()
        fill(0.8)
        ellipse(self.position.x, self.position.y, self.radius * 2, self.radius * 2)


run()
