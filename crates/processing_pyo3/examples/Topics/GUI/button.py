# Button.
#
# Click on one of the colored shapes in the
# center of the image to change the color of
# the background.
from mewnala import *

rect_x = 0  # Position of square button
rect_y = 0
circle_x = 0  # Position of circle button
circle_y = 0
rect_size = 90  # Diameter of rect
circle_size = 93  # Diameter of circle
rect_color = None
circle_color = None
base_color = None
rect_highlight = None
circle_highlight = None
current_color = None
rect_over = False
circle_over = False


def setup():
    global rect_color, rect_highlight, circle_color, circle_highlight, base_color, current_color
    global circle_x, circle_y, rect_x, rect_y
    size(640, 360)
    rect_color = color(0.0)
    rect_highlight = color(0.2)
    circle_color = color(1.0)
    circle_highlight = color(0.8)
    base_color = color(0.4)
    current_color = base_color
    circle_x = width // 2 + circle_size // 2 + 10
    circle_y = height // 2
    rect_x = width // 2 - rect_size - 10
    rect_y = height // 2 - rect_size // 2
    ellipse_mode(CENTER)


def draw():
    update(mouse_x, mouse_y)
    background(current_color)

    if rect_over:
        fill(rect_highlight)
    else:
        fill(rect_color)
    stroke(1.0)
    rect(rect_x, rect_y, rect_size, rect_size)

    if circle_over:
        fill(circle_highlight)
    else:
        fill(circle_color)
    stroke(0.0)
    ellipse(circle_x, circle_y, circle_size, circle_size)


def update(x, y):
    global circle_over, rect_over
    if over_circle(circle_x, circle_y, circle_size):
        circle_over = True
        rect_over = False
    elif over_rect(rect_x, rect_y, rect_size, rect_size):
        rect_over = True
        circle_over = False
    else:
        circle_over = rect_over = False


def mouse_pressed():
    global current_color
    if circle_over:
        current_color = circle_color
    if rect_over:
        current_color = rect_color


def over_rect(x, y, w, h):
    if x <= mouse_x <= x + w and y <= mouse_y <= y + h:
        return True
    else:
        return False


def over_circle(x, y, diameter):
    dis_x = x - mouse_x
    dis_y = y - mouse_y
    if sqrt(sq(dis_x) + sq(dis_y)) < diameter / 2:
        return True
    else:
        return False


run()
