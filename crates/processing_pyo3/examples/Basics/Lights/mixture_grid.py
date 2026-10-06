# Mixture Grid
# modified from an example by Simon Greenwold.
#
# Display a 2D grid of boxes with three different kinds of lights.
from mewnala import *

point = None
directional = None
spot = None


def setup():
    size(640, 360)
    mode_3d()
    no_stroke()
    define_lights()


def draw():
    background(0.0)

    for x in range(0, width + 1, 60):
        for y in range(0, height + 1, 60):
            push_matrix()
            translate(x - width / 2, height / 2 - y, 0)
            rotate_y(remap(mouse_x, 0, width, 0, PI))
            rotate_x(remap(mouse_y, 0, height, 0, PI))
            box(90)
            pop_matrix()


def define_lights():
    global point, directional, spot
    # The lights are created once, placed relative to the top-left corner
    # as in the original (the world is centered and y-up)
    # Orange point light on the right
    point = point_light((0.59, 0.39, 0.0), 3000000000.0, 600.0, 1.0,  # Color
                        position=(200 - width / 2, height / 2 + 150, 0))  # Position

    # Blue directional light from the left
    directional = directional_light((0.0, 0.4, 1.0), 4000.0,  # Color
                                    look_at=(1, 0, 0))  # The x-, y-, z-axis direction

    # Yellow spotlight from the front
    spot = spot_light((1.0, 1.0, 0.43), 2000000000.0, 600.0, 1.0,  # Color
                      PI / 4, PI / 2,  # Angle, concentration
                      position=(-width / 2, height / 2 - 40, 200),  # Position
                      look_at=(-width / 2, height / 2 - 40 + 0.5, 200 - 0.5))  # Direction
    roughness(0.6)


run()
