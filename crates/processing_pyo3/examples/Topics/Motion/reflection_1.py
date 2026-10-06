# Non-orthogonal Reflection
# by Ira Greenberg.
#
# Based on the equation (R = 2N(N*L)-L) where R is the
# reflection vector, N is the normal, and L is the incident
# vector.
from mewnala import *

# Position of left hand side of floor
base1 = None
# Position of right hand side of floor
base2 = None
# Length of floor
base_length = 0.0

# A list of subpoints along the floor path
coords = []

# Variables related to moving ball
position = None
velocity = None
r = 6
speed = 3.5


def setup():
    global base1, base2, position, velocity
    size(640, 360)

    fill(0.5)
    base1 = Vec2(0, height - 150)
    base2 = Vec2(width, height)
    create_ground()

    # start ellipse at middle top of screen
    position = Vec2(width / 2, 0)

    # calculate initial random velocity
    velocity = Vec2.random()
    velocity.mult(speed)


def draw():
    # draw background
    fill(0.0, 0.05)
    no_stroke()
    rect(0, 0, width, height)

    # draw base
    fill(0.78)
    quad(base1.x, base1.y, base2.x, base2.y, base2.x, height, 0, height)

    # calculate base top normal
    base_delta = (base2 - base1).normalize()
    normal = Vec2(-base_delta.y, base_delta.x)

    # draw ellipse
    no_stroke()
    fill(1.0)
    ellipse(position.x, position.y, r * 2, r * 2)

    # move ellipse
    position.add(velocity)

    # normalized incidence vector
    incidence = (velocity * -1).normalize()

    # detect and handle collision
    for i in range(len(coords)):
        # check distance between ellipse and base top coordinates
        if position.dist(coords[i]) < r:
            # calculate dot product of incident vector and base top normal
            dot = incidence.dot(normal)

            # calculate reflection vector
            # assign reflection vector to direction vector
            velocity.set(2 * normal.x * dot - incidence.x, 2 * normal.y * dot - incidence.y)
            velocity.mult(speed)

            # draw base top normal at collision point
            stroke(1.0, 0.5, 0.0)
            line(position.x, position.y, position.x - normal.x * 100, position.y - normal.y * 100)

    # detect boundary collision
    # right
    if position.x > width - r:
        position.x = width - r
        velocity.x *= -1
    # left
    if position.x < r:
        position.x = r
        velocity.x *= -1
    # top
    if position.y < r:
        position.y = r
        velocity.y *= -1
        # randomize base top
        base1.y = random(height - 100, height)
        base2.y = random(height - 100, height)
        create_ground()


# Calculate variables for the ground
def create_ground():
    global base_length, coords
    # calculate length of base top
    base_length = base1.dist(base2)

    # fill base top coordinate list
    coords = []
    for i in range(int(ceil(base_length))):
        c = Vec2()
        c.x = base1.x + ((base2.x - base1.x) / base_length) * i
        c.y = base1.y + ((base2.y - base1.y) / base_length) * i
        coords.append(c)


run()
