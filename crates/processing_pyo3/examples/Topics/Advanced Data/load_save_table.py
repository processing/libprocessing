# Loading Tabular Data
# by Daniel Shiffman.
#
# This example demonstrates how to use the csv module
# to retrieve data from a CSV file and make objects
# from that data.
#
# Here is what the CSV looks like:
#
# x,y,diameter,name
# 160,103,43.19838,Happy
# 372,137,52.42526,Sad
# 273,235,61.14072,Joyous
# 121,179,44.758068,Melancholy
from mewnala import *
import csv
import os

# A list of Bubble objects
bubbles = []
# The table: a list of rows, each row a dict keyed by column name
table = []
# open() resolves against the cwd, so build the path from this file
csv_path = os.path.join(os.path.dirname(__file__), "data", "data.csv")


def setup():
    size(640, 360)
    load_data()


def draw():
    background(1.0)
    # Display all bubbles
    for b in bubbles:
        b.display()
        b.rollover(mouse_x, mouse_y)

    text_align(LEFT)
    fill(0.0)
    text("Click to add bubbles.", 10, height - 10)


def load_data():
    global bubbles, table
    # Load CSV file into a list of rows
    # DictReader uses the header row for the column names
    with open(csv_path, newline="") as f:
        table = list(csv.DictReader(f))

    # The list of Bubble objects is built from every row in the CSV
    bubbles = []

    # You can iterate over all the rows in a table
    for row in table:
        # You can access the fields via their column name
        x = float(row["x"])
        y = float(row["y"])
        d = float(row["diameter"])
        n = row["name"]
        # Make a Bubble object out of the data read
        bubbles.append(Bubble(x, y, d, n))


def mouse_pressed():
    # Create a new row
    row = {}
    # Set the values of that row
    row["x"] = mouse_x
    row["y"] = mouse_y
    row["diameter"] = random(40, 80)
    row["name"] = "Blah"
    table.append(row)

    # If the table has more than 10 rows
    if len(table) > 10:
        # Delete the oldest row
        table.pop(0)

    # Writing the CSV back to the same file
    with open(csv_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["x", "y", "diameter", "name"])
        writer.writeheader()
        writer.writerows(table)
    # And reloading it
    load_data()


# A Bubble class
class Bubble:
    def __init__(self, x, y, diameter, s):
        self.x = x
        self.y = y
        self.diameter = diameter
        self.name = s
        self.over = False

    # Checking if mouse is over the Bubble
    def rollover(self, px, py):
        d = dist(px, py, self.x, self.y)
        if d < self.diameter / 2:
            self.over = True
        else:
            self.over = False

    # Display the Bubble
    def display(self):
        stroke(0.0)
        stroke_weight(2)
        no_fill()
        ellipse(self.x, self.y, self.diameter, self.diameter)
        if self.over:
            fill(0.0)
            text_align(CENTER)
            text(self.name, self.x, self.y + self.diameter / 2 + 20)


run()
