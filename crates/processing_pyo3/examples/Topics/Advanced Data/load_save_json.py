# Loading JSON Data
# by Daniel Shiffman.
#
# This example demonstrates how to use the json module
# to retrieve data from a JSON file and make objects
# from that data.
#
# Here is what the JSON looks like (partial):
#
# {
#   "bubbles": [
#     {
#       "position": {
#         "x": 160,
#         "y": 103
#       },
#       "diameter": 43.19838,
#       "label": "Happy"
#     },
#     {
#       "position": {
#         "x": 372,
#         "y": 137
#       },
#       "diameter": 52.42526,
#       "label": "Sad"
#     }
#   ]
# }
from mewnala import *
import json
import os

# A list of Bubble objects
bubbles = []
# The JSON data
data = None
# open() resolves against the cwd, so build the path from this file
json_path = os.path.join(os.path.dirname(__file__), "data", "data.json")


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
    global bubbles, data
    # Load JSON file
    with open(json_path) as f:
        data = json.load(f)

    bubble_data = data["bubbles"]

    # The list of Bubble objects is built from every object in the "bubbles" array
    bubbles = []

    for bubble in bubble_data:
        # Get a position object
        position = bubble["position"]
        # Get x,y from position
        x = position["x"]
        y = position["y"]

        # Get diameter and label
        diameter = bubble["diameter"]
        label = bubble["label"]

        # Put object in list
        bubbles.append(Bubble(x, y, diameter, label))


def mouse_pressed():
    # Create a new JSON bubble object
    new_bubble = {}

    # Create a new JSON position object
    position = {}
    position["x"] = int(mouse_x)
    position["y"] = int(mouse_y)

    # Add position to bubble
    new_bubble["position"] = position

    # Add diameter and label to bubble
    new_bubble["diameter"] = random(40, 80)
    new_bubble["label"] = "New label"

    # Append the new JSON bubble object to the array
    bubble_data = data["bubbles"]
    bubble_data.append(new_bubble)

    if len(bubble_data) > 10:
        bubble_data.pop(0)

    # Save new data
    with open(json_path, "w") as f:
        json.dump(data, f, indent=2)
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
