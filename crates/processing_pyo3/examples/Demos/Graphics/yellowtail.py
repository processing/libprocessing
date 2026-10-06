# Yellowtail
# by Golan Levin (www.flong.com).
#
# Click, drag, and release to create a kinetic gesture.
#
# Yellowtail (1998-2000) is an interactive software system for the gestural
# creation and performance of real-time abstract animation. Yellowtail repeats
# a user's strokes end-over-end, enabling simultaneous specification of a
# line's shape and quality of movement. Each line repeats according to its
# own period, producing an ever-changing and responsive display of lively,
# worm-like textures.
from mewnala import *

gesture_array = []
n_gestures = 36  # Number of gestures
min_move = 3  # Minimum travel for a new point
current_gesture_id = -1


def setup():
    global current_gesture_id
    size(1024, 768)
    background(0.0, 0.0, 0.0)
    no_stroke()

    current_gesture_id = -1
    for i in range(n_gestures):
        gesture_array.append(Gesture(width, height))
    clear_gestures()


def draw():
    background(0.0)

    update_geometry()
    fill(1.0, 1.0, 0.96)
    for i in range(n_gestures):
        render_gesture(gesture_array[i], width, height)


def mouse_pressed():
    global current_gesture_id
    current_gesture_id = (current_gesture_id + 1) % n_gestures
    g = gesture_array[current_gesture_id]
    g.clear()
    g.clear_polys()
    g.add_point(mouse_x, mouse_y)


def mouse_dragged():
    if current_gesture_id >= 0:
        g = gesture_array[current_gesture_id]
        if g.dist_to_last(mouse_x, mouse_y) > min_move:
            g.add_point(mouse_x, mouse_y)
            g.smooth()
            g.compile()


def key_pressed():
    if key == "+" or key == "=":
        if current_gesture_id >= 0:
            th = gesture_array[current_gesture_id].thickness
            gesture_array[current_gesture_id].thickness = min(96, th + 1)
            gesture_array[current_gesture_id].compile()
    elif key == "-":
        if current_gesture_id >= 0:
            th = gesture_array[current_gesture_id].thickness
            gesture_array[current_gesture_id].thickness = max(2, th - 1)
            gesture_array[current_gesture_id].compile()
    elif key == " ":
        clear_gestures()


def render_gesture(gesture, w, h):
    if gesture.exists:
        if gesture.n_polys > 0:
            polygons = gesture.polygons
            crosses = gesture.crosses

            begin_shape(QUADS)
            gnp = gesture.n_polys
            for i in range(gnp):
                p = polygons[i]
                xpts = p.xpoints
                ypts = p.ypoints

                vertex(xpts[0], ypts[0])
                vertex(xpts[1], ypts[1])
                vertex(xpts[2], ypts[2])
                vertex(xpts[3], ypts[3])

                cr = crosses[i]
                if cr > 0:
                    if (cr & 3) > 0:
                        vertex(xpts[0] + w, ypts[0])
                        vertex(xpts[1] + w, ypts[1])
                        vertex(xpts[2] + w, ypts[2])
                        vertex(xpts[3] + w, ypts[3])

                        vertex(xpts[0] - w, ypts[0])
                        vertex(xpts[1] - w, ypts[1])
                        vertex(xpts[2] - w, ypts[2])
                        vertex(xpts[3] - w, ypts[3])
                    if (cr & 12) > 0:
                        vertex(xpts[0], ypts[0] + h)
                        vertex(xpts[1], ypts[1] + h)
                        vertex(xpts[2], ypts[2] + h)
                        vertex(xpts[3], ypts[3] + h)

                        vertex(xpts[0], ypts[0] - h)
                        vertex(xpts[1], ypts[1] - h)
                        vertex(xpts[2], ypts[2] - h)
                        vertex(xpts[3], ypts[3] - h)

                    # I have knowingly retained the small flaw of not
                    # completely dealing with the corner conditions
                    # (the case in which both of the above are true).
            end_shape()


def update_geometry():
    for g in range(n_gestures):
        j = gesture_array[g]
        if j.exists:
            if g != current_gesture_id:
                advance_gesture(j)
            elif not mouse_is_pressed:
                advance_gesture(j)


def advance_gesture(gesture):
    # Move a Gesture one step
    if gesture.exists:  # check
        n_pts = gesture.n_points
        n_pts1 = n_pts - 1
        jx = gesture.jump_dx
        jy = gesture.jump_dy

        if n_pts > 0:
            path = gesture.path
            for i in range(n_pts1, 0, -1):
                path[i].x = path[i - 1].x
                path[i].y = path[i - 1].y
            path[0].x = path[n_pts1].x - jx
            path[0].y = path[n_pts1].y - jy
            gesture.compile()


def clear_gestures():
    for i in range(n_gestures):
        gesture_array[i].clear()


# Stands in for java.awt.Polygon: four integer corners
class Polygon:
    def __init__(self):
        self.npoints = 0
        self.xpoints = [0, 0, 0, 0]
        self.ypoints = [0, 0, 0, 0]


class Gesture:
    def __init__(self, mw, mh):
        self.damp = 5.0
        self.damp_inv = 1.0 / self.damp
        self.damp1 = self.damp - 1

        self.w = mw
        self.h = mh
        self.capacity = 600
        self.path = []
        self.polygons = []
        self.crosses = []
        for i in range(self.capacity):
            self.polygons.append(Polygon())
            self.polygons[i].npoints = 4
            self.path.append(Vec3f())
            self.crosses.append(0)
        self.n_points = 0
        self.n_polys = 0

        self.exists = False
        self.jump_dx = 0
        self.jump_dy = 0
        self.INIT_TH = 14
        self.thickness = self.INIT_TH

    def clear(self):
        self.n_points = 0
        self.exists = False
        self.thickness = self.INIT_TH

    def clear_polys(self):
        self.n_polys = 0

    def add_point(self, x, y):
        if self.n_points >= self.capacity:
            # there are all sorts of possible solutions here,
            # but for abject simplicity, I don't do anything.
            pass
        else:
            v = self.dist_to_last(x, y)
            p = self.get_pressure_from_velocity(v)
            self.path[self.n_points].set(x, y, p)
            self.n_points += 1

            if self.n_points > 1:
                self.exists = True
                self.jump_dx = self.path[self.n_points - 1].x - self.path[0].x
                self.jump_dy = self.path[self.n_points - 1].y - self.path[0].y

    def get_pressure_from_velocity(self, v):
        scale = 18
        min_p = 0.02
        old_p = self.path[self.n_points - 1].p if self.n_points > 0 else 0
        return ((min_p + max(0, 1.0 - v / scale)) + (self.damp1 * old_p)) * self.damp_inv

    def set_pressures(self):
        # pressures vary from 0...1
        t = 0
        u = 1.0 / (self.n_points - 1) * TWO_PI
        for i in range(self.n_points):
            pressure = sqrt((1.0 - cos(t)) * 0.5)
            self.path[i].p = pressure
            t += u

    def dist_to_last(self, ix, iy):
        if self.n_points > 0:
            v = self.path[self.n_points - 1]
            dx = v.x - ix
            dy = v.y - iy
            return mag(dx, dy)
        else:
            return 30

    def compile(self):
        # compute the polygons from the path of Vec3f's
        if self.exists:
            self.clear_polys()

            taper = 1.0

            n_path_points = self.n_points - 1
            last_poly_index = n_path_points - 1
            npm1finv = 1.0 / max(1, n_path_points - 1)

            # handle the first point
            p0 = self.path[0]
            p1 = self.path[1]
            radius0 = p0.p * self.thickness
            dx01 = p1.x - p0.x
            dy01 = p1.y - p0.y
            hp01 = sqrt(dx01 * dx01 + dy01 * dy01)
            if hp01 == 0:
                hp02 = 0.0001
            co01 = radius0 * dx01 / hp01
            si01 = radius0 * dy01 / hp01
            ax = p0.x - si01
            ay = p0.y + co01
            bx = p0.x + si01
            by = p0.y - co01

            LC = 20
            RC = self.w - LC
            TC = 20
            BC = self.h - TC
            mint = 0.618
            tapow = 0.4

            # handle the middle points
            for i in range(1, n_path_points):
                taper = pow((last_poly_index - i) * npm1finv, tapow)

                p0 = self.path[i - 1]
                p1 = self.path[i]
                p2 = self.path[i + 1]
                p1x = p1.x
                p1y = p1.y
                radius1 = max(mint, taper * p1.p * self.thickness)

                # assumes all segments are roughly the same length...
                dx02 = p2.x - p0.x
                dy02 = p2.y - p0.y
                hp02 = sqrt(dx02 * dx02 + dy02 * dy02)
                if hp02 != 0:
                    hp02 = radius1 / hp02
                co02 = dx02 * hp02
                si02 = dy02 * hp02

                # translate the integer coordinates to the viewing rectangle
                axi = axip = int(ax)
                ayi = ayip = int(ay)
                axi = (self.w - ((-axi) % self.w)) if axi < 0 else axi % self.w
                axid = axi - axip
                ayi = (self.h - ((-ayi) % self.h)) if ayi < 0 else ayi % self.h
                ayid = ayi - ayip

                # set the vertices of the polygon
                apoly = self.polygons[self.n_polys]
                self.n_polys += 1
                xpts = apoly.xpoints
                ypts = apoly.ypoints
                cx = p1x + si02
                dx = p1x - si02
                cy = p1y - co02
                dy = p1y + co02
                xpts[0] = axi = axid + axip
                xpts[1] = bxi = axid + int(bx)
                xpts[2] = cxi = axid + int(cx)
                xpts[3] = dxi = axid + int(dx)
                ypts[0] = ayi = ayid + ayip
                ypts[1] = byi = ayid + int(by)
                ypts[2] = cyi = ayid + int(cy)
                ypts[3] = dyi = ayid + int(dy)

                # keep a record of where we cross the edge of the screen
                self.crosses[i] = 0
                if axi <= LC or bxi <= LC or cxi <= LC or dxi <= LC:
                    self.crosses[i] |= 1
                if axi >= RC or bxi >= RC or cxi >= RC or dxi >= RC:
                    self.crosses[i] |= 2
                if ayi <= TC or byi <= TC or cyi <= TC or dyi <= TC:
                    self.crosses[i] |= 4
                if ayi >= BC or byi >= BC or cyi >= BC or dyi >= BC:
                    self.crosses[i] |= 8

                # swap data for next time
                ax = dx
                ay = dy
                bx = cx
                by = cy

            # handle the last point
            p2 = self.path[n_path_points]
            apoly = self.polygons[self.n_polys]
            self.n_polys += 1
            xpts = apoly.xpoints
            ypts = apoly.ypoints

            xpts[0] = int(ax)
            xpts[1] = int(bx)
            xpts[2] = int(p2.x)
            xpts[3] = int(p2.x)

            ypts[0] = int(ay)
            ypts[1] = int(by)
            ypts[2] = int(p2.y)
            ypts[3] = int(p2.y)

    def smooth(self):
        # average neighboring points
        weight = 18
        scale = 1.0 / (weight + 2)
        n_points_minus_two = self.n_points - 2

        for i in range(1, n_points_minus_two):
            lower = self.path[i - 1]
            center = self.path[i]
            upper = self.path[i + 1]

            center.x = (lower.x + weight * center.x + upper.x) * scale
            center.y = (lower.y + weight * center.y + upper.y) * scale


class Vec3f:
    def __init__(self, ix=0, iy=0, ip=0):
        self.set(ix, iy, ip)

    def set(self, ix, iy, ip):
        self.x = ix
        self.y = iy
        self.p = ip  # Pressure


run()
