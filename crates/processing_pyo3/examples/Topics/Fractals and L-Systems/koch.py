# Koch Curve
# by Daniel Shiffman.
#
# Renders a simple fractal, the Koch snowflake.
# Each recursive level is drawn in sequence.
from mewnala import *

k = None


def setup():
    global k
    size(640, 360)
    frame_rate(1)  # Animate slowly
    k = KochFractal()


def draw():
    background(0.0)
    # Draws the snowflake!
    k.render()
    # Iterate
    k.next_level()
    # Let's not do it more than 5 times. . .
    if k.get_count() > 5:
        k.restart()


# Koch Curve
# A class to manage the list of line segments in the snowflake pattern
class KochFractal:
    def __init__(self):
        self.start = Vec2(0, height - 20)  # A Vec2 for the start
        self.end = Vec2(width, height - 20)  # A Vec2 for the end
        self.lines = []  # A list to keep track of all the lines
        self.count = 0
        self.restart()

    def next_level(self):
        # For every line that is in the list
        # create 4 more lines in a new list
        self.lines = self.iterate(self.lines)
        self.count += 1

    def restart(self):
        self.count = 0  # Reset count
        self.lines.clear()  # Empty the list
        self.lines.append(KochLine(self.start, self.end))  # Add the initial line (from one end Vec2 to the other)

    def get_count(self):
        return self.count

    # This is easy, just draw all the lines
    def render(self):
        for l in self.lines:
            l.display()

    # This is where the **MAGIC** happens
    # Step 1: Create an empty list
    # Step 2: For every line currently in the list
    #   - calculate 4 line segments based on Koch algorithm
    #   - add all 4 line segments into the new list
    # Step 3: Return the new list and it becomes the list of line segments for the structure

    # As we do this over and over again, each line gets broken into 4 lines, which gets broken into 4 lines, and so on. . .
    def iterate(self, before):
        now = []  # Create empty list
        for l in before:
            # Calculate 5 koch Vec2s (done for us by the line object)
            a = l.start()
            b = l.kochleft()
            c = l.kochmiddle()
            d = l.kochright()
            e = l.end()
            # Make line segments between all the Vec2s and add them
            now.append(KochLine(a, b))
            now.append(KochLine(b, c))
            now.append(KochLine(c, d))
            now.append(KochLine(d, e))
        return now


# The Nature of Code
# Daniel Shiffman
# http://natureofcode.com

# Koch Curve
# A class to describe one line segment in the fractal
# Includes methods to calculate midpoints along the line according to the Koch algorithm
class KochLine:
    # Two Vec2s,
    # a is the "left" Vec2 and
    # b is the "right" Vec2
    def __init__(self, start, end):
        self.a = start.copy()
        self.b = end.copy()

    def display(self):
        stroke(1.0)
        line(self.a.x, self.a.y, self.b.x, self.b.y)

    def start(self):
        return self.a.copy()

    def end(self):
        return self.b.copy()

    # This is easy, just 1/3 of the way
    def kochleft(self):
        v = self.b - self.a
        v.div(3)
        v.add(self.a)
        return v

    # More complicated, have to use a little trig to figure out where this Vec2 is!
    def kochmiddle(self):
        v = self.b - self.a
        v.div(3)

        p = self.a.copy()
        p.add(v)

        v = v.rotate(-radians(60))
        p.add(v)

        return p

    # Easy, just 2/3 of the way
    def kochright(self):
        v = self.a - self.b
        v.div(3)
        v.add(self.b)
        return v


run()
