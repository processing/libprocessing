# Pie Chart
#
# Uses the arc() function to generate a pie chart from the data
# stored in an array.
from mewnala import *

angles = [30, 10, 45, 35, 60, 38, 75, 67]


def setup():
    size(640, 360)
    no_stroke()
    no_loop()  # Run once and stop


def draw():
    background(0.39)
    pie_chart(300, angles)


def pie_chart(diameter, data):
    last_angle = 0
    for i in range(len(data)):
        gray = remap(i, 0, len(data), 0, 1)
        fill(gray)
        arc(width / 2, height / 2, diameter, diameter, last_angle, last_angle + radians(data[i]))
        last_angle += radians(data[i])


run()
