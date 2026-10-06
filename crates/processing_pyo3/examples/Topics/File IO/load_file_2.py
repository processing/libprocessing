# LoadFile 2
#
# This example loads a data file about cars. Each element is separated
# with a tab and corresponds to a different aspect of each car. The file stores
# the miles per gallon, cylinders, displacement, etc., for more than 400 different
# makes and models. Press a mouse button to advance to the next group of entries.
from mewnala import *
import os

records = []
lines = []
record_count = 0
body = None
num = 9  # Display this many entries on each screen.
starting_entry = 0  # Display from this entry number


def setup():
    global lines, record_count, body
    size(640, 360)
    fill(1.0)
    no_loop()

    body = create_font("Helvetica")
    text_font(body)
    text_size(20)

    # open() resolves against the cwd, so build the path from this file
    path = os.path.join(os.path.dirname(__file__), "data", "cars2.tsv")
    with open(path) as f:
        lines = f.read().splitlines()
    for i in range(len(lines)):
        pieces = lines[i].split("\t")  # Load data into list
        if len(pieces) == 9:
            records.append(Record(pieces))
            record_count += 1


def draw():
    background(0.0)
    for i in range(num):
        this_entry = starting_entry + i
        if this_entry < record_count:
            text(str(this_entry) + " > " + records[this_entry].name, 20, 20 + i * 20)


def mouse_pressed():
    global starting_entry
    starting_entry += num
    if starting_entry > len(records):
        starting_entry = 0  # go back to the beginning
    redraw()


# Processing's float() gives NaN for a string that isn't a number (the data has "NA" entries)
def parse_float(s):
    try:
        return float(s)
    except ValueError:
        return float("nan")


class Record:
    def __init__(self, pieces):
        self.name = pieces[0]
        self.mpg = parse_float(pieces[1])
        self.cylinders = int(pieces[2])
        self.displacement = parse_float(pieces[3])
        self.horsepower = parse_float(pieces[4])
        self.weight = parse_float(pieces[5])
        self.acceleration = parse_float(pieces[6])
        self.year = int(pieces[7])
        self.origin = parse_float(pieces[8])


run()
