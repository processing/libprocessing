# ArrayList of objects
# by Daniel Shiffman.
#
# This example demonstrates how to use a list to store
# a variable number of objects.  Items can be added and removed
# from the list.
#
# Click the mouse to add bouncing balls.
from mewnala import *

balls = []
ball_width = 48


def setup():
    size(640, 360)
    no_stroke()

    # Start by adding one element
    balls.append(Ball(width / 2, 0, ball_width))


def draw():
    background(1.0)

    # The length of a list is dynamic
    # Notice how we are looping through the list backwards
    # This is because we are deleting elements from the list
    for i in range(len(balls) - 1, -1, -1):
        ball = balls[i]
        ball.move()
        ball.display()
        if ball.finished():
            # Items can be deleted with pop()
            balls.pop(i)


def mouse_pressed():
    # A new ball object is added to the list (by default to the end)
    balls.append(Ball(mouse_x, mouse_y, ball_width))


# Simple bouncing ball class
class Ball:
    def __init__(self, temp_x, temp_y, temp_w):
        self.x = temp_x
        self.y = temp_y
        self.w = temp_w
        self.speed = 0
        self.gravity = 0.1
        self.life = 255

    def move(self):
        # Add gravity to speed
        self.speed = self.speed + self.gravity
        # Add speed to y location
        self.y = self.y + self.speed
        # If square reaches the bottom
        # Reverse speed
        if self.y > height:
            # Dampening
            self.speed = self.speed * -0.8
            self.y = height

    def finished(self):
        # Balls fade out
        self.life -= 1
        if self.life < 0:
            return True
        else:
            return False

    def display(self):
        # Display the circle
        fill(0.0, self.life / 255)
        ellipse(self.x, self.y, self.w, self.w)


run()
