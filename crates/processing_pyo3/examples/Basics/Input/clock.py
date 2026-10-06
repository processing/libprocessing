# Clock.
#
# The current time can be read with Python's datetime module.
# In this example, sin() and cos() values are used to set the
# position of the hands.
from datetime import datetime
from mewnala import *

cx = 0
cy = 0
seconds_radius = 0.0
minutes_radius = 0.0
hours_radius = 0.0
clock_diameter = 0.0


def setup():
    global cx, cy, seconds_radius, minutes_radius, hours_radius, clock_diameter
    size(640, 360)
    stroke(1.0)

    radius = min(width, height) / 2
    seconds_radius = radius * 0.72
    minutes_radius = radius * 0.60
    hours_radius = radius * 0.50
    clock_diameter = radius * 1.8

    cx = width / 2
    cy = height / 2


def draw():
    background(0.0)

    # Draw the clock background
    fill(0.31)
    no_stroke()
    ellipse(cx, cy, clock_diameter, clock_diameter)

    now = datetime.now()

    # Angles for sin() and cos() start at 3 o'clock;
    # subtract HALF_PI to make them start at the top
    s = remap(now.second, 0, 60, 0, TWO_PI) - HALF_PI
    m = remap(now.minute + norm(now.second, 0, 60), 0, 60, 0, TWO_PI) - HALF_PI
    h = remap(now.hour + norm(now.minute, 0, 60), 0, 24, 0, TWO_PI * 2) - HALF_PI

    # Draw the hands of the clock
    stroke(1.0)
    stroke_weight(1)
    line(cx, cy, cx + cos(s) * seconds_radius, cy + sin(s) * seconds_radius)
    stroke_weight(2)
    line(cx, cy, cx + cos(m) * minutes_radius, cy + sin(m) * minutes_radius)
    stroke_weight(4)
    line(cx, cy, cx + cos(h) * hours_radius, cy + sin(h) * hours_radius)

    # Draw the minute ticks
    stroke_weight(2)
    begin_shape(POINTS)
    for a in range(0, 360, 6):
        angle = radians(a)
        x = cx + cos(angle) * seconds_radius
        y = cy + sin(angle) * seconds_radius
        vertex(x, y)
    end_shape()


run()
