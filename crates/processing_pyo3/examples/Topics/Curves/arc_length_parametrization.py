# Arc Length parametrization of curves by Jakub Valtar
#
# This example shows how to divide a curve into segments
# of an equal length and how to move along the curve with
# constant speed.
#
# To demonstrate the technique, a cubic Bezier curve is used.
# However, this technique is applicable to any kind of
# parametric curve.
from mewnala import *
from bisect import bisect_left

curve = None

points = []
equidistant_points = []

t = 0.0
t_step = 0.004

POINT_COUNT = 80

border_size = 40


def setup():
    global curve, points, equidistant_points
    size(640, 360)

    frame_rate(60)
    text_align(CENTER)
    text_size(16)
    stroke_weight(2)

    a = Vec2(0, 300)
    b = Vec2(440, 0)
    c = Vec2(-200, 0)
    d = Vec2(240, 300)

    curve = BezierCurve(a, b, c, d)

    points = curve.points(POINT_COUNT)
    equidistant_points = curve.equidistant_points(POINT_COUNT)


def draw():
    global t

    # Show static value when mouse is pressed, animate otherwise
    if mouse_is_pressed:
        a = constrain(mouse_x, border_size, width - border_size)
        t = remap(a, border_size, width - border_size, 0.0, 1.0)
    else:
        t += t_step
        if t > 1.0:
            t = 0.0

    background(1.0)

    # draw curve and circle using standard parametrization
    push_matrix()
    translate(border_size, -50)

    label_style()
    text("STANDARD\nPARAMETRIZATION", 120, 310)

    curve_style()
    begin_shape(LINES)
    for i in range(0, len(points) - 1, 2):
        vertex(points[i].x, points[i].y)
        vertex(points[i + 1].x, points[i + 1].y)
    end_shape()

    circle_style()
    pos1 = curve.point_at_parameter(t)
    ellipse(pos1.x, pos1.y, 12, 12)

    pop_matrix()

    # draw curve and circle using arc length parametrization
    push_matrix()
    translate(width / 2 + border_size, -50)

    label_style()
    text("ARC LENGTH\nPARAMETRIZATION", 120, 310)

    curve_style()
    begin_shape(LINES)
    for i in range(0, len(equidistant_points) - 1, 2):
        vertex(equidistant_points[i].x, equidistant_points[i].y)
        vertex(equidistant_points[i + 1].x, equidistant_points[i + 1].y)
    end_shape()

    circle_style()
    pos2 = curve.point_at_fraction(t)
    ellipse(pos2.x, pos2.y, 12, 12)

    pop_matrix()

    # draw seek bar
    push_matrix()
    translate(border_size, height - 45)

    bar_length = width - 2 * border_size

    bar_bg_style()
    line(0, 0, bar_length, 0)
    line(bar_length, -5, bar_length, 5)

    bar_style()
    line(0, -5, 0, 5)
    line(0, 0, t * bar_length, 0)

    bar_label_style()
    text("{:.2f}".format(t), bar_length / 2, 25)
    pop_matrix()


# Styles -----

def curve_style():
    stroke(0.67)
    no_fill()


def label_style():
    no_stroke()
    fill(0.47)


def circle_style():
    no_stroke()
    fill(0.0)


def bar_bg_style():
    stroke(0.86)
    no_fill()


def bar_style():
    stroke(0.2)
    no_fill()


def bar_label_style():
    no_stroke()
    fill(0.47)


# This class represents a cubic Bezier curve.
#
# point_at_parameter() method works the same as bezier_point().
#
# Points returned from this method are closer to each other
# at places where the curve bends and farther apart where the
# curve runs straight.
#
# On the other hand, point_at_fraction() and point_at_length()
# return points at fixed distances. This is useful in many scenarios:
# you may want to move an object along the curve at some speed
# or you may want to draw dashed Bezier curves.
class BezierCurve:
    SEGMENT_COUNT = 100

    def __init__(self, a, b, c, d):
        self.v0 = a.copy()  # curve begins here
        self.v1 = b.copy()
        self.v2 = c.copy()
        self.v3 = d.copy()  # curve ends here

        self.arc_lengths = [0.0] * (self.SEGMENT_COUNT + 1)  # there are n segments between n+1 points

        # The idea here is to make a handy look up table, which contains
        # parameter values with their arc lengths along the curve. Later,
        # when we want a point at some arc length, we can go through our
        # table, pick the place where the point is going to be located and
        # interpolate the value of parameter from two surrounding parameters
        # in our table.

        # we will keep current length along the curve here
        arc_length = 0

        prev = Vec2()
        prev.set(self.v0)

        # i goes from 0 to SEGMENT_COUNT
        for i in range(self.SEGMENT_COUNT + 1):

            # map index from range (0, SEGMENT_COUNT) to parameter in range (0.0, 1.0)
            t = i / self.SEGMENT_COUNT

            # get point on the curve at this parameter value
            point = self.point_at_parameter(t)

            # get distance from previous point
            distance_from_prev = prev.dist(point)

            # add arc length of last segment to total length
            arc_length += distance_from_prev

            # save current arc length to the look up table
            self.arc_lengths[i] = arc_length

            # keep this point to compute length of next segment
            prev.set(point)

        # Here we have sum of all segment lengths, which should be
        # very close to the actual length of the curve. The more
        # segments we use, the more accurate it becomes.
        self.curve_length = arc_length

    # Returns the length of this curve
    def length(self):
        return self.curve_length

    # Returns a point along the curve at a specified parameter value.
    def point_at_parameter(self, t):
        result = Vec2()
        result.x = bezier_point(self.v0.x, self.v1.x, self.v2.x, self.v3.x, t)
        result.y = bezier_point(self.v0.y, self.v1.y, self.v2.y, self.v3.y, t)
        return result

    # Returns a point at a fraction of curve's length.
    # Example: point_at_fraction(0.25) returns point at one quarter of curve's length.
    def point_at_fraction(self, r):
        wanted_length = self.curve_length * r
        return self.point_at_length(wanted_length)

    # Returns a point at a specified arc length along the curve.
    def point_at_length(self, wanted_length):
        wanted_length = constrain(wanted_length, 0.0, self.curve_length)

        # look up the length in our look up table
        index = bisect_left(self.arc_lengths, wanted_length)

        if index == len(self.arc_lengths) or self.arc_lengths[index] != wanted_length:
            # exact length is not in the table, but bisect tells us
            # where it should be in the table

            # interpolate two surrounding indexes
            next_index = index
            prev_index = next_index - 1
            prev_length = self.arc_lengths[prev_index]
            next_length = self.arc_lengths[next_index]
            mapped_index = remap(wanted_length, prev_length, next_length, prev_index, next_index)

        else:
            # wanted length is in the table, we know the index right away
            mapped_index = index

        # map index from range (0, SEGMENT_COUNT) to parameter in range (0.0, 1.0)
        parameter = mapped_index / self.SEGMENT_COUNT

        return self.point_at_parameter(parameter)

    # Returns a list of equidistant points on the curve
    def equidistant_points(self, how_many):

        result_points = [None] * how_many

        # we already know the beginning and the end of the curve
        result_points[0] = self.v0.copy()
        result_points[how_many - 1] = self.v3.copy()

        arc_length_index = 1
        for i in range(1, how_many - 1):

            # compute wanted arc length
            fraction = i / (how_many - 1)
            wanted_length = fraction * self.curve_length

            # move through the look up table until we find greater length
            while wanted_length > self.arc_lengths[arc_length_index] and arc_length_index < len(self.arc_lengths):
                arc_length_index += 1

            # interpolate two surrounding indexes
            next_index = arc_length_index
            prev_index = arc_length_index - 1
            prev_length = self.arc_lengths[prev_index]
            next_length = self.arc_lengths[next_index]
            mapped_index = remap(wanted_length, prev_length, next_length, prev_index, next_index)

            # map index from range (0, SEGMENT_COUNT) to parameter in range (0.0, 1.0)
            parameter = mapped_index / self.SEGMENT_COUNT

            result_points[i] = self.point_at_parameter(parameter)

        return result_points

    # Returns a list of points on the curve.
    def points(self, how_many):

        result_points = [None] * how_many

        # we already know the first and the last point of the curve
        result_points[0] = self.v0.copy()
        result_points[how_many - 1] = self.v3.copy()

        for i in range(1, how_many - 1):

            # map index to parameter in range (0.0, 1.0)
            parameter = i / (how_many - 1)

            result_points[i] = self.point_at_parameter(parameter)

        return result_points


run()
