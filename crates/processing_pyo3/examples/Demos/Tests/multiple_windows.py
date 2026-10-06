# Multiple Windows
#
# Based on code by GeneKao (https://github.com/GeneKao)
# A second window is opened with create_window(); it is drawn through
# its own Graphics methods inside draw(), and its mouse is polled.
from mewnala import *

child = None
mouse_pressed_on_parent = False
arcball = None
arcball2 = None


def setup():
    global child, arcball, arcball2
    size(320, 240)
    mode_3d()
    window_title("Main sketch")
    arcball = Arcball(None, 300)
    child = ChildApplet()
    arcball2 = Arcball(child.g, 300)


def draw():
    global mouse_pressed_on_parent
    background(0.98)
    arcball.run()
    if mouse_is_pressed:
        fill(0.0)
        text("Mouse pressed on parent.", 10 - width / 2, height / 2 - 10)
        fill(0.0, 0.94, 0.0)
        ellipse(mouse_x - width / 2, height / 2 - mouse_y, 60, 60)
        mouse_pressed_on_parent = True
    else:
        fill(0.08)
        ellipse(0, 0, 60, 60)
        mouse_pressed_on_parent = False
    box(100)
    if child.g.mouse_is_pressed:
        text("Mouse pressed on child.", 10 - width / 2, height / 2 - 30)

    child.draw()


def mouse_pressed():
    arcball.mouse_pressed()


def mouse_dragged():
    arcball.mouse_dragged()


class ChildApplet:
    def __init__(self):
        self.g = create_window(400, 400, "Child sketch")
        self.g.mode_3d()
        self.was_pressed = False

    def draw(self):
        g = self.g
        # Secondary windows have no event callbacks: poll the mouse instead
        if g.mouse_is_pressed and not self.was_pressed:
            arcball2.mouse_pressed()
        elif g.mouse_is_pressed:
            arcball2.mouse_dragged()
        self.was_pressed = g.mouse_is_pressed

        g.background(0.0)
        arcball2.run()
        if g.mouse_is_pressed:
            g.fill(0.94, 0.0, 0.0)
            g.ellipse(g.mouse_x - g.width / 2, g.height / 2 - g.mouse_y, 20, 20)
            g.fill(1.0)
            g.text("Mouse pressed on child.", 10 - g.width / 2, g.height / 2 - 30)
        else:
            g.fill(1.0)
            g.ellipse(0, 0, 20, 20)

        g.draw_box(100, 200, 100)
        if mouse_pressed_on_parent:
            g.fill(1.0)
            g.text("Mouse pressed on parent", 20 - g.width / 2, g.height / 2 - 20)


# Ariel and V3ga's arcball class with a couple tiny mods by Robert Hodgin
# `parent` is the Graphics of the window it drives (None for the main window).
class Arcball:
    def __init__(self, parent, radius):
        self.parent = parent
        self.radius = radius
        self.center_x = 0
        self.center_y = 0

        self.v_down = Vec3()
        self.v_drag = Vec3()

        self.q_now = Quat()
        self.q_down = Quat()
        self.q_drag = Quat()

        self.axis_set = [Vec3(1.0, 0.0, 0.0), Vec3(0.0, 1.0, 0.0), Vec3(0.0, 0.0, 1.0)]
        self.axis = -1  # no constraints...
        self.mxv = 0
        self.myv = 0
        self.x = 0
        self.y = 0

    def p_mouse_x(self):
        if self.parent is None:
            return mouse_x
        return self.parent.mouse_x

    def p_mouse_y(self):
        if self.parent is None:
            return mouse_y
        return self.parent.mouse_y

    def mouse_pressed(self):
        self.v_down = self.mouse_to_sphere(self.p_mouse_x(), self.p_mouse_y())
        self.q_down.set(self.q_now)
        self.q_drag.reset()

    def mouse_dragged(self):
        self.v_drag = self.mouse_to_sphere(self.p_mouse_x(), self.p_mouse_y())
        self.q_drag.set_axis(self.v_down.dot(self.v_drag), self.v_down.cross(self.v_drag))

    def run(self):
        if self.parent is None:
            self.center_x = width / 2.0
            self.center_y = height / 2.0
        else:
            self.center_x = self.parent.width / 2.0
            self.center_y = self.parent.height / 2.0

        self.q_now = Quat.mul(self.q_drag, self.q_down)
        # The 3D origin is already the canvas center, so no translate is needed
        self.apply_quat_2_matrix(self.q_now)

        self.x += self.mxv
        self.y += self.myv
        self.mxv -= self.mxv * 0.01
        self.myv -= self.myv * 0.01

    def mouse_to_sphere(self, x, y):
        v = Vec3()
        v.x = (x - self.center_x) / self.radius
        v.y = (y - self.center_y) / self.radius

        m = v.x * v.x + v.y * v.y
        if m > 1.0:
            v.normalize()
        else:
            v.z = sqrt(1.0 - m)

        return v if self.axis == -1 else self.constrain_vector(v, self.axis_set[self.axis])

    def constrain_vector(self, vector, axis):
        res = vector - axis * axis.dot(vector)
        res.normalize()
        return res

    def apply_quat_2_matrix(self, q):
        # instead of transforming q into a matrix and applying it...
        aa = q.get_value()
        # A zero axis (no rotation yet) is not accepted by rotate_axis()
        if aa[1] == 0 and aa[2] == 0 and aa[3] == 0:
            return
        if self.parent is None:
            rotate_axis(aa[0], aa[1], aa[2], aa[3])
        else:
            self.parent.rotate_axis(aa[0], aa[1], aa[2], aa[3])


class Quat:
    def __init__(self, w=1.0, x=0.0, y=0.0, z=0.0):
        self.w = w
        self.x = x
        self.y = y
        self.z = z

    def reset(self):
        self.w = 1.0
        self.x = 0.0
        self.y = 0.0
        self.z = 0.0

    def set_axis(self, w, v):
        self.w = w
        self.x = v.x
        self.y = v.y
        self.z = v.z

    def set(self, q):
        self.w = q.w
        self.x = q.x
        self.y = q.y
        self.z = q.z

    @staticmethod
    def mul(q1, q2):
        res = Quat()
        res.w = q1.w * q2.w - q1.x * q2.x - q1.y * q2.y - q1.z * q2.z
        res.x = q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y
        res.y = q1.w * q2.y + q1.y * q2.w + q1.z * q2.x - q1.x * q2.z
        res.z = q1.w * q2.z + q1.z * q2.w + q1.x * q2.y - q1.y * q2.x
        return res

    def get_value(self):
        # transforming this quat into an angle and an axis vector...
        sa = sqrt(max(0.0, 1.0 - self.w * self.w))
        if sa < EPSILON:
            sa = 1.0

        return [acos(constrain(self.w, -1.0, 1.0)) * 2.0, self.x / sa, self.y / sa, self.z / sa]


EPSILON = 0.0001

run()
