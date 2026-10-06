# Loading XML Data
# by Daniel Shiffman.
#
# This example demonstrates how to use xml.etree
# to retrieve data from an XML file and make objects
# from that data.
#
# Here is what the XML looks like:
#
# <?xml version="1.0"?>
# <bubbles>
#   <bubble>
#     <position x="160" y="103"/>
#     <diameter>43.19838</diameter>
#     <label>Happy</label>
#   </bubble>
#   <bubble>
#     <position x="372" y="137"/>
#     <diameter>52.42526</diameter>
#     <label>Sad</label>
#   </bubble>
# </bubbles>
from mewnala import *
import os
import xml.etree.ElementTree as ET

# A list of Bubble objects
bubbles = []
# The XML root element
xml = None
# open() resolves against the cwd, so build the path from this file
xml_path = os.path.join(os.path.dirname(__file__), "data", "data.xml")


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
    global bubbles, xml
    # Load XML file
    xml = ET.parse(xml_path).getroot()
    # Get all the child nodes named "bubble"
    children = xml.findall("bubble")

    # The list of Bubble objects is built from every XML element named "bubble"
    bubbles = []

    for child in children:
        # The position element has two attributes: x and y
        position_element = child.find("position")
        # Attributes are strings, so convert them to numbers
        x = int(position_element.get("x"))
        y = int(position_element.get("y"))

        # The diameter is the content of the child named "diameter"
        diameter_element = child.find("diameter")
        # The content of an XML node is its text, converted to a number
        diameter = float(diameter_element.text)

        # The label is the content of the child named "label"
        label_element = child.find("label")
        label = label_element.text

        # Make a Bubble object out of the data read
        bubbles.append(Bubble(x, y, diameter, label))


def mouse_pressed():
    # Create a new XML bubble element
    bubble = ET.SubElement(xml, "bubble")

    # Set the position element
    position = ET.SubElement(bubble, "position")
    # Attributes are set as strings
    position.set("x", str(int(mouse_x)))
    position.set("y", str(int(mouse_y)))

    # Set the diameter element
    diameter = ET.SubElement(bubble, "diameter")
    # Here for a node's content, we have to convert to a string
    diameter.text = str(random(40, 80))

    # Set a label
    label = ET.SubElement(bubble, "label")
    label.text = "New label"

    # Here we are removing the oldest bubble if there are more than 10
    children = xml.findall("bubble")
    # If the XML file has more than 10 bubble elements
    if len(children) > 10:
        # Delete the first one
        xml.remove(children[0])

    # Save a new XML file
    ET.indent(xml)
    ET.ElementTree(xml).write(xml_path, encoding="UTF-8", xml_declaration=True)

    # reload the new data
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
