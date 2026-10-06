# Edge Filter
#
# Apply a custom shader to the filter() function to affect the geometry drawn to the screen.
#
# Press the mouse to turn the filter on and off.
from mewnala import *

edges = None
apply_filter = True


def setup():
    global edges
    size(640, 360)
    mode_3d()
    edges = load_shader("data/edges.wesl")
    no_stroke()
    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)


def draw():
    background(0.0)

    push_matrix()
    rotate_x(frame_count * 0.01)
    rotate_y(frame_count * 0.01)
    box(120)
    pop_matrix()

    if apply_filter:
        filter(edges)

    # The sphere doesn't have the edge detection applied
    # on it because it is drawn after filter() is called.
    rotate_y(frame_count * 0.02)
    translate(150, 0)
    sphere(40)


def mouse_pressed():
    global apply_filter
    apply_filter = not apply_filter


run()
