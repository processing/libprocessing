# Polar to Cartesian
# by Daniel Shiffman.
#
# Convert a polar coordinate (r,theta) to cartesian (x,y).
# The calculations are x=r*cos(theta) and y=r*sin(theta).
from mewnala import *

r = 0.0

# Angle and angular velocity, acceleration
theta = 0.0
theta_vel = 0.0
theta_acc = 0.0


def setup():
    global r, theta, theta_vel, theta_acc
    size(640, 360)

    # Initialize all values
    r = height * 0.45
    theta = 0
    theta_vel = 0
    theta_acc = 0.0001


def draw():
    global theta, theta_vel

    background(0.0)

    # Translate the origin point to the center of the screen
    translate(width / 2, height / 2)

    # Convert polar to cartesian
    x = r * cos(theta)
    y = r * sin(theta)

    # Draw the ellipse at the cartesian coordinate
    ellipse_mode(CENTER)
    no_stroke()
    fill(0.78)
    ellipse(x, y, 32, 32)

    # Apply acceleration and velocity to angle
    theta_vel += theta_acc
    theta += theta_vel


run()
