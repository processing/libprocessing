# Text Rotation.
#
# Draws letters to the screen and rotates them at different angles.
from mewnala import *

f = None
angle_rotate = 0.0


def setup():
    global f
    size(640, 360)
    background(0.0)

    # Create the font from the .ttf file in the data folder
    f = load_font("data/SourceCodePro-Regular.ttf")
    text_font(f)
    text_size(18)


def draw():
    global angle_rotate
    background(0.0)

    stroke_weight(1)
    stroke(0.6)

    push_matrix()
    angle1 = radians(45)
    translate(100, 180)
    rotate(angle1)
    text("45 DEGREES", 0, 0)
    line(0, 0, 150, 0)
    pop_matrix()

    push_matrix()
    angle2 = radians(270)
    translate(200, 180)
    rotate(angle2)
    text("270 DEGREES", 0, 0)
    line(0, 0, 150, 0)
    pop_matrix()

    push_matrix()
    translate(440, 180)
    rotate(radians(angle_rotate))
    text(str(int(angle_rotate) % 360) + " DEGREES", 0, 0)
    line(0, 0, 150, 0)
    pop_matrix()

    angle_rotate += 0.25

    stroke(1.0, 0.0, 0.0)
    stroke_weight(4)
    point(100, 180)
    point(200, 180)
    point(440, 180)


run()
