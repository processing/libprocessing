# Wolfram Cellular Automata
# by Daniel Shiffman.
#
# Simple demonstration of a Wolfram's 1-dimensional
# cellular automata. When the system reaches bottom
# of the window, it restarts with a new ruleset.
# Mouse click restarts as well.
from mewnala import *

ca = None  # An instance object to the cellular automata


def setup():
    global ca
    size(640, 360)
    ruleset = [0, 1, 0, 1, 1, 0, 1, 0]  # An initial rule system
    ca = CA(ruleset)  # Initialize CA
    background(0.0)


def draw():
    ca.render()  # Draw the CA
    ca.generate()  # Generate the next level

    # If we're done, clear the screen,
    # pick a new ruleset and restart
    if ca.finished():
        background(0.0)
        ca.randomize()
        ca.restart()


def mouse_pressed():
    background(0.0)
    ca.randomize()
    ca.restart()


class CA:
    def __init__(self, r):
        self.rules = r  # Array to store the rules, for example [0,1,1,0,1,1,0,1]
        self.scl = 1  # How many pixels wide/high is each cell?
        self.cells = [0] * (width // self.scl)  # An array of 0s and 1s
        self.generation = 0  # How many generations?
        self.restart()

    # Set the rules of the CA
    def set_rules(self, r):
        self.rules = r

    # Make a random ruleset
    def randomize(self):
        for i in range(8):
            self.rules[i] = int(random(2))

    # Reset to generation 0
    def restart(self):
        for i in range(len(self.cells)):
            self.cells[i] = 0
        # We arbitrarily start with just the middle
        # cell having a state of "1"
        self.cells[len(self.cells) // 2] = 1
        self.generation = 0

    # The process of creating the new generation
    def generate(self):
        # First we create an empty array for the new values
        nextgen = [0] * len(self.cells)
        # For every spot, determine new state by examing current
        # state, and neighbor states
        # Ignore edges that only have one neighor
        for i in range(1, len(self.cells) - 1):
            left = self.cells[i - 1]  # Left neighbor state
            me = self.cells[i]  # Current state
            right = self.cells[i + 1]  # Right neighbor state
            # Compute next generation state based on ruleset
            nextgen[i] = self.execute_rules(left, me, right)
        # Copy the array into current value
        for i in range(1, len(self.cells) - 1):
            self.cells[i] = nextgen[i]
        self.generation += 1

    # This is the easy part, just draw the cells,
    # fill 1.0 for '1', fill 0.0 for '0'
    def render(self):
        for i in range(len(self.cells)):
            if self.cells[i] == 1:
                fill(1.0)
            else:
                fill(0.0)
            no_stroke()
            rect(i * self.scl, self.generation * self.scl, self.scl, self.scl)

    # Implementing the Wolfram rules
    # Could be improved and made more concise,
    # but here we can explicitly see what is going on for each case
    def execute_rules(self, a, b, c):
        if a == 1 and b == 1 and c == 1:
            return self.rules[0]
        if a == 1 and b == 1 and c == 0:
            return self.rules[1]
        if a == 1 and b == 0 and c == 1:
            return self.rules[2]
        if a == 1 and b == 0 and c == 0:
            return self.rules[3]
        if a == 0 and b == 1 and c == 1:
            return self.rules[4]
        if a == 0 and b == 1 and c == 0:
            return self.rules[5]
        if a == 0 and b == 0 and c == 1:
            return self.rules[6]
        if a == 0 and b == 0 and c == 0:
            return self.rules[7]
        return 0

    # The CA is done if it reaches the bottom of the screen
    def finished(self):
        if self.generation > height // self.scl:
            return True
        else:
            return False


run()
