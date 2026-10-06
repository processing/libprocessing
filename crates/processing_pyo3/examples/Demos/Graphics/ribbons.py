# Ribbons, by Andres Colubri
# ArcBall class by Ariel, V3ga and Robert Hodgin (flight404)
#
# This sketch loads 3D atomic coordinates of a protein molecule
# from a file in PDB format (http://www.pdb.org/) and displays
# the structure using a ribbon representation.
from mewnala import *
import os

pdb_file = "data/4HHB.pdb"  # PDB file to read
# pdb_file = "data/1CBS.pdb"
# pdb_file = "data/2POR.pdb"

# Some parameters to control the visual appearance:
scale_factor = 10  # Size factor
render_mode = 1  # 0 = lines, 1 = flat ribbons
ribbon_detail = 4  # Ribbon detail: from 1 (lowest) to 4 (highest)
helix_diam = 10  # Helix diameter.
ribbon_width = [10, 7, 2]  # Ribbon widths for helix, strand and coil
ribbon_color = color(0.0, 0.4, 0.6, 1.0)  # Ribbon color

# All the molecular models read from the PDB file (it could contain more than one)
models = []
# Translation that centers the models at (0, 0, 0)
model_shift = None

arcball = None


def setup():
    global arcball
    size(1024, 768)
    mode_3d()

    arcball = Arcball(width / 2, height / 2, 600)
    read_pdb(pdb_file)

    # ambient(80); lights()
    directional_light((1.0, 1.0, 1.0), 4000.0, position=(0, 0, 1), look_at=(0, 0, 0))
    roughness(0.6)


def draw():
    background(0.0)

    translate(0, 0, 200)
    arcball.run()

    translate(model_shift.x, model_shift.y, model_shift.z)
    no_stroke()
    for i in range(len(models)):
        draw_geometry(models[i])


def mouse_pressed():
    arcball.mouse_pressed()


def mouse_dragged():
    arcball.mouse_dragged()


# Ariel and V3ga's arcball class with a couple tiny mods by Robert Hodgin
class Arcball:
    def __init__(self, center_x, center_y, radius):
        self.center_x = center_x
        self.center_y = center_y
        self.radius = radius

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

    def mouse_pressed(self):
        self.v_down = self.mouse_to_sphere(mouse_x, mouse_y)
        self.q_down.set(self.q_now)
        self.q_drag.reset()

    def mouse_dragged(self):
        self.v_drag = self.mouse_to_sphere(mouse_x, mouse_y)
        self.q_drag.set_axis(self.v_down.dot(self.v_drag), self.v_down.cross(self.v_drag))

    def run(self):
        self.q_now = Quat.mul(self.q_drag, self.q_down)
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
        rotate_axis(aa[0], aa[1], aa[2], aa[3])


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


# ---------------------------------------------------------------- BSpline

MAX_BEZIER_ORDER = 10  # Maximum curve order.

BSPLINE_MATRIX = [
    [-1.0 / 6.0, 1.0 / 2.0, -1.0 / 2.0, 1.0 / 6.0],
    [1.0 / 2.0, -1.0, 1.0 / 2.0, 0.0],
    [-1.0 / 2.0, 0.0, 1.0 / 2.0, 0.0],
    [1.0 / 6.0, 2.0 / 3.0, 1.0 / 6.0, 0.0],
]

# The element(i, n) of this array contains the binomial coefficient
# C(i, n) = n!/(i!(n-i)!)
BINOMIAL_COEF_TABLE = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    [0, 1, 3, 6, 10, 15, 21, 28, 36, 45],
    [0, 0, 1, 4, 10, 20, 35, 56, 84, 120],
    [0, 0, 0, 1, 5, 15, 35, 70, 126, 210],
    [0, 0, 0, 0, 1, 6, 21, 56, 126, 252],
    [0, 0, 0, 0, 0, 1, 7, 28, 84, 210],
    [0, 0, 0, 0, 0, 0, 1, 8, 36, 120],
    [0, 0, 0, 0, 0, 0, 0, 1, 9, 45],
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 10],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
]

# The element of this(i, j) of this table contains(i/10)^(3-j).
T_VECTOR_TABLE = [
    #  t^3,  t^2, t^1, t^0
    [0, 0, 0, 1],  # t = 0.0
    [0.001, 0.01, 0.1, 1],  # t = 0.1
    [0.008, 0.04, 0.2, 1],  # t = 0.2
    [0.027, 0.09, 0.3, 1],  # t = 0.3
    [0.064, 0.16, 0.4, 1],  # t = 0.4
    [0.125, 0.25, 0.5, 1],  # t = 0.5
    [0.216, 0.36, 0.6, 1],  # t = 0.6
    [0.343, 0.49, 0.7, 1],  # t = 0.7
    [0.512, 0.64, 0.8, 1],  # u = 0.8
    [0.729, 0.81, 0.9, 1],  # t = 0.9
    [1, 1, 1, 1],  # t = 1.0
]

# The element of this(i, j) of this table contains(3-j)*(i/10)^(2-j) if
# j < 3, 0 otherwise.
DT_VECTOR_TABLE = [
    # 3t^2,  2t^1, t^0
    [0, 0, 1, 0],  # t = 0.0
    [0.03, 0.2, 1, 0],  # t = 0.1
    [0.12, 0.4, 1, 0],  # t = 0.2
    [0.27, 0.6, 1, 0],  # t = 0.3
    [0.48, 0.8, 1, 0],  # t = 0.4
    [0.75, 1.0, 1, 0],  # t = 0.5
    [1.08, 1.2, 1, 0],  # t = 0.6
    [1.47, 1.4, 1, 0],  # t = 0.7
    [1.92, 1.6, 1, 0],  # t = 0.8
    [2.43, 1.8, 1, 0],  # t = 0.9
    [3, 2, 1, 0],  # t = 1.0
]


class Spline:
    # The factorial of n.
    def factorial(self, n):
        return 1 if n <= 0 else n * self.factorial(n - 1)

    # Gives n!/(i!(n-i)!).
    def binomial_coef(self, i, n):
        if i <= MAX_BEZIER_ORDER and n <= MAX_BEZIER_ORDER:
            return BINOMIAL_COEF_TABLE[i][n - 1]
        else:
            return int(self.factorial(n) / (self.factorial(i) * self.factorial(n - i)))

    # Evaluates the Berstein polinomial(i, n) at u.
    def berstein_pol(self, i, n, u):
        return self.binomial_coef(i, n) * pow(u, i) * pow(1 - u, n - i)

    # The derivative of the Berstein polinomial.
    def dberstein_pol(self, i, n, u):
        if i == 0:
            s1 = 0
        else:
            s1 = i * pow(u, i - 1) * pow(1 - u, n - i)
        if n == i:
            s2 = 0
        else:
            s2 = -(n - i) * pow(u, i) * pow(1 - u, n - i - 1)
        return self.binomial_coef(i, n) * (s1 + s2)


class BSpline(Spline):
    def __init__(self, t=True):
        self.init_parameters(t)

    # Sets lookup table use.
    def init_parameters(self, t):
        self.bspline_c_points = [[0.0, 0.0, 0.0] for i in range(4)]
        self.t_vector = [0.0] * 4
        self.dt_vector = [0.0] * 4
        self.m3 = [[0.0, 0.0, 0.0] for i in range(4)]
        self.pt = [0.0] * 3
        self.tg = [0.0] * 3
        self.lookup = t

    # Sets n-th control point.
    def set_c_point(self, n, p):
        self.bspline_c_points[n][0] = p.x
        self.bspline_c_points[n][1] = p.y
        self.bspline_c_points[n][2] = p.z
        self.update_matrix3()

    # Gets n-th control point.
    def get_c_point(self, n, p):
        p.set(self.bspline_c_points[n][0], self.bspline_c_points[n][1], self.bspline_c_points[n][2])

    # Replaces the current B-spline control points(0, 1, 2) with(1, 2, 3). This
    # is used when a new spline is to be joined to the recently drawn.
    def shift_bspline_c_points(self):
        for i in range(3):
            self.bspline_c_points[0][i] = self.bspline_c_points[1][i]
            self.bspline_c_points[1][i] = self.bspline_c_points[2][i]
            self.bspline_c_points[2][i] = self.bspline_c_points[3][i]
        self.update_matrix3()

    def copy_c_points(self, n_source, n_dest):
        for i in range(3):
            self.bspline_c_points[n_dest][i] = self.bspline_c_points[n_source][i]

    # Updates the temporal matrix used in order 3 calculations.
    def update_matrix3(self):
        for i in range(4):
            for j in range(3):
                s = 0
                for k in range(4):
                    s += BSPLINE_MATRIX[i][k] * self.bspline_c_points[k][j]
                self.m3[i][j] = s

    def feval(self, t, p):
        self.eval_point(t)
        p.set(self.pt[0], self.pt[1], self.pt[2])

    def deval(self, t, d):
        self.eval_tangent(t)
        d.set(self.tg[0], self.tg[1], self.tg[2])

    def feval_x(self, t):
        self.eval_point(t)
        return self.pt[0]

    def feval_y(self, t):
        self.eval_point(t)
        return self.pt[1]

    def feval_z(self, t):
        self.eval_point(t)
        return self.pt[2]

    def deval_x(self, t):
        self.eval_tangent(t)
        return self.tg[0]

    def deval_y(self, t):
        self.eval_tangent(t)
        return self.tg[1]

    def deval_z(self, t):
        self.eval_tangent(t)
        return self.tg[2]

    # Point evaluation.
    def eval_point(self, t):
        if self.lookup:
            self.bspline_point_i(int(10 * t))
        else:
            self.bspline_point(t)

    # Tangent evaluation.
    def eval_tangent(self, t):
        if self.lookup:
            self.bspline_tangent_i(int(10 * t))
        else:
            self.bspline_tangent(t)

    # Calculates the point on the cubic spline corresponding to the parameter value t in [0, 1].
    def bspline_point(self, t):
        # Q(u) = UVector * BSplineMatrix * BSplineCPoints
        for i in range(4):
            self.t_vector[i] = pow(t, 3 - i)

        for j in range(3):
            s = 0
            for k in range(4):
                s += self.t_vector[k] * self.m3[k][j]
            self.pt[j] = s

    # Calculates the tangent vector of the spline at t.
    def bspline_tangent(self, t):
        # Q(u) = DTVector * BSplineMatrix * BSplineCPoints
        for i in range(4):
            if i < 3:
                self.dt_vector[i] = (3 - i) * pow(t, 2 - i)
            else:
                self.dt_vector[i] = 0

        for j in range(3):
            s = 0
            for k in range(4):
                s += self.dt_vector[k] * self.m3[k][j]
            self.tg[j] = s

    # Gives the point on the cubic spline corresponding to t/10(using the lookup table).
    def bspline_point_i(self, t):
        # Q(u) = TVectorTable[u] * BSplineMatrix * BSplineCPoints
        for j in range(3):
            s = 0
            for k in range(4):
                s += T_VECTOR_TABLE[t][k] * self.m3[k][j]
            self.pt[j] = s

    # Calulates the tangent vector of the spline at t/10.
    def bspline_tangent_i(self, t):
        # Q(u) = DTVectorTable[u] * BSplineMatrix * BSplineCPoints
        for j in range(3):
            s = 0
            for k in range(4):
                s += DT_VECTOR_TABLE[t][k] * self.m3[k][j]
            self.tg[j] = s


# ---------------------------------------------------------------- Geometry

spline_side1 = None
spline_center = None
spline_side2 = None
flip_test_v = None
uspacing = 1

HELIX = 0
STRAND = 1
COIL = 2
LHANDED = -1
RHANDED = 1


def create_ribbon_model(residues, trj):
    global uspacing, flip_test_v, spline_side1, spline_center, spline_side2

    # For line ribbons
    vertices0 = []
    vertices1 = []
    vertices2 = []

    # For flat ribbons
    vertices = []
    normals = []

    if ribbon_detail == 1:
        uspacing = 10
    elif ribbon_detail == 2:
        uspacing = 5
    elif ribbon_detail == 3:
        uspacing = 2
    else:
        uspacing = 1

    flip_test_v = Vec3()
    spline_side1 = BSpline(False)
    spline_center = BSpline(False)
    spline_side2 = BSpline(False)

    ss = [0] * len(residues)
    handness = [0] * len(residues)

    calculate_sec_str(residues, ss, handness)

    for i in range(len(residues)):
        construct_control_points(residues, i, ss[i], handness[i])

        if render_mode == 0:
            generate_spline(0, vertices0)
            generate_spline(1, vertices1)
            generate_spline(2, vertices2)
        else:
            generate_flat_ribbon(vertices, normals)

    if render_mode == 0:
        # Three polylines, one geometry each
        for line in (vertices0, vertices1, vertices2):
            model = create_geometry(topology=LINE_STRIP)
            for pos_vec in line:
                model.color(ribbon_color.r, ribbon_color.g, ribbon_color.b, ribbon_color.a)
                model.vertex(pos_vec.x, pos_vec.y, pos_vec.z)
            trj.append(model)
    else:
        # The ribbon construction is fairly inneficient here, since
        # it could use triangle strips instead to avoid duplicating
        # shared vertices...
        model = create_geometry(topology=TRIANGLES)
        for i in range(len(vertices)):
            pos_vec = vertices[i]
            norm_vec = normals[i]
            model.color(ribbon_color.r, ribbon_color.g, ribbon_color.b, ribbon_color.a)
            model.normal(-norm_vec.x, -norm_vec.y, -norm_vec.z)
            model.vertex(pos_vec.x, pos_vec.y, pos_vec.z)
        trj.append(model)

    if render_mode == 0:
        tot_count = len(vertices0) + len(vertices1) + len(vertices2)
        print("Adding new model with " + str(tot_count) + " vertices.")
    else:
        print("Adding new model with " + str(len(vertices)) + " vertices.")


def calculate_gyr_radius(atoms):
    r = 0
    for i in range(len(atoms)):
        ati = atoms[i]
        for j in range(i + 1, len(atoms)):
            atj = atoms[j]

            dx = ati.x - atj.x
            dy = ati.y - atj.y
            dz = ati.z - atj.z
            r += dx * dx + dy * dy + dz * dz
    return sqrt(r) / (len(atoms) + 1)


# Does a cheap and dirty secondary structure assignment to the protein
# residues given in the array.
def calculate_sec_str(residues, ss, handness):
    n = len(residues)

    phi = [0.0] * n
    psi = [0.0] * n

    for i in range(n):
        if i == 0 or i == n - 1:
            phi[i] = 90
            psi[i] = 90
        else:
            res0 = residues[i - 1]
            res1 = residues[i]
            res2 = residues[i + 1]

            c0 = res0["C"]
            n1 = res1["N"]
            ca1 = res1["CA"]
            c1 = res1["C"]
            n2 = res2["N"]

            phi[i] = calculate_torsional_angle(c0, n1, ca1, c1)
            psi[i] = calculate_torsional_angle(n1, ca1, c1, n2)

    first_helix = 0
    ncons_r_helix = 0
    ncons_l_helix = 0
    first_strand = 0
    ncons_strand = 0
    for i in range(n):
        # Right-handed helix
        if dist(phi[i], psi[i], -60, -45) < 30 and i < n - 1:
            if ncons_r_helix == 0:
                first_helix = i
            ncons_r_helix += 1
        else:
            if 3 <= ncons_r_helix:
                for k in range(first_helix, i):
                    ss[k] = HELIX
                    handness[k] = RHANDED
            ncons_r_helix = 0

        # Left-handed helix
        if dist(phi[i], psi[i], +60, +45) < 30 and i < n - 1:
            if ncons_l_helix == 0:
                first_helix = i
            ncons_l_helix += 1
        else:
            if 3 <= ncons_l_helix:
                for k in range(first_helix, i):
                    ss[k] = HELIX
                    handness[k] = LHANDED
            ncons_l_helix = 0

        # Strand
        if dist(phi[i], psi[i], -110, +130) < 30 and i < n - 1:
            if ncons_strand == 0:
                first_strand = i
            ncons_strand += 1
        else:
            if 2 <= ncons_strand:
                for k in range(first_strand, i):
                    ss[k] = STRAND
                    handness[k] = RHANDED
            ncons_strand = 0

        ss[i] = COIL
        handness[i] = RHANDED


# Calculates the torsional angle defined by four atoms with positions at0, at1, at2 and at3.
def calculate_torsional_angle(at0, at1, at2, at3):
    r01 = at0 - at1
    r32 = at3 - at2
    r12 = at1 - at2

    p = r12.cross(r01)
    q = r12.cross(r32)
    r = r12.cross(q)

    u = q.dot(q)
    v = r.dot(r)

    if u <= 0.0 or v <= 0.0:
        a = 360.0
    else:
        u1 = p.dot(q)  # u1 = p * q
        v1 = p.dot(r)  # v1 = p * r

        u = u1 / sqrt(u)
        v = v1 / sqrt(v)

        if abs(u) > 0.01 or abs(v) > 0.01:
            a = degrees(atan2(v, u))
        else:
            a = 360.0
    return a


def generate_spline(n, vertices):
    v1 = Vec3()

    if n == 0:
        spline_side1.feval(0, v1)
    elif n == 1:
        spline_center.feval(0, v1)
    else:
        spline_side2.feval(0, v1)
    vertices.append(Vec3(v1.x, v1.y, v1.z))

    for ui in range(1, 11):
        if ui % uspacing == 0:
            u = 0.1 * ui

            if n == 0:
                spline_side1.feval(u, v1)
            elif n == 1:
                spline_center.feval(u, v1)
            else:
                spline_side2.feval(u, v1)

            vertices.append(Vec3(v1.x, v1.y, v1.z))


def generate_flat_ribbon(vertices, normals):
    cent_point0 = Vec3()
    cent_point1 = Vec3()
    sid1_point0 = Vec3()
    sid1_point1 = Vec3()
    sid2_point0 = Vec3()
    sid2_point1 = Vec3()
    tangent = Vec3()
    normal0 = Vec3()

    # The initial geometry is generated.
    spline_side1.feval(0, sid1_point1)
    spline_center.feval(0, cent_point1)
    spline_side2.feval(0, sid2_point1)

    # The tangents at the three previous points are the same.
    spline_side2.deval(0, tangent)

    # Vector transversal to the ribbon.
    transversal = sid1_point1 - sid2_point1

    # The normal is calculated.
    normal1 = transversal.cross(tangent)
    normal1.normalize()

    for ui in range(1, 11):
        if ui % uspacing == 0:
            u = 0.1 * ui

            # The geometry of the previous iteration is saved.
            sid1_point0.set(sid1_point1)
            cent_point0.set(cent_point1)
            sid2_point0.set(sid2_point1)
            normal0.set(normal1)

            # The new geometry is generated.
            spline_side1.feval(u, sid1_point1)
            spline_center.feval(u, cent_point1)
            spline_side2.feval(u, sid2_point1)

            # The tangents at the three previous points are the same.
            spline_side2.deval(u, tangent)
            # Vector transversal to the ribbon.
            transversal = sid1_point1 - sid2_point1
            # The normal is calculated.
            normal1 = transversal.cross(tangent)
            normal1.normalize()

            # The (Sid1Point0, Sid1Point1, CentPoint1) triangle is added.
            vertices.append(cent_point1.copy())
            normals.append(normal1.copy())

            vertices.append(sid1_point1.copy())
            normals.append(normal1.copy())

            vertices.append(sid1_point0.copy())
            normals.append(normal0.copy())

            # The (Sid1Point0, CentPoint1, CentPoint0) triangle is added.
            vertices.append(cent_point0.copy())
            normals.append(normal0.copy())

            vertices.append(cent_point1.copy())
            normals.append(normal1.copy())

            vertices.append(sid1_point0.copy())
            normals.append(normal0.copy())

            # (Sid2Point0, Sid2Point1, CentPoint1) triangle is added.
            vertices.append(sid2_point0.copy())
            normals.append(normal0.copy())

            vertices.append(sid2_point1.copy())
            normals.append(normal1.copy())

            vertices.append(cent_point1.copy())
            normals.append(normal1.copy())

            # (Sid2Point0, CentPoint1, CentPoint0) triangle is added.
            vertices.append(sid2_point0.copy())
            normals.append(normal0.copy())

            vertices.append(cent_point1.copy())
            normals.append(normal1.copy())

            vertices.append(cent_point0.copy())
            normals.append(normal0.copy())


# The code in the following three functions is based in the method introduced
# in this paper:
# "Algorithm for ribbon models of proteins."
# Authors: Mike Carson and Charles E. Bugg
# Published in: J.Mol.Graphics 4, pp. 121-122 (1986)

# Shifts the control points one place to the left.
def shift_control_points():
    spline_side1.shift_bspline_c_points()
    spline_center.shift_bspline_c_points()
    spline_side2.shift_bspline_c_points()


# Adds a new control point to the arrays CPCenter, CPRight and CPLeft
def add_control_points(ca0, ox0, ca1, ss, handness):
    a = ca1 - ca0
    b = ox0 - ca0

    # Vector normal to the peptide plane (pointing outside in the case of the
    # alpha helix).
    c = a.cross(b)

    # Vector contained in the peptide plane (perpendicular to its direction).
    d = c.cross(a)

    # Normalizing vectors.
    c.normalize()
    d.normalize()

    # Flipping test (to avoid self crossing in the strands).
    if ss != HELIX and 90.0 < degrees(Vec3.angle_between(flip_test_v, d)):
        # Flip detected. The plane vector is inverted.
        d.mult(-1.0)

    # The central control point is constructed.
    cpt0 = linear_comb(0.5, ca0, 0.5, ca1)
    spline_center.set_c_point(3, cpt0)

    if ss == HELIX:
        # When residue i is contained in a helix, the control point is moved away
        # from the helix axis, along the C direction.
        p0 = Vec3()
        spline_center.get_c_point(3, p0)
        cpt0 = linear_comb(1.0, p0, handness * helix_diam, c)
        spline_center.set_c_point(3, cpt0)

    # The control points for the side ribbons are constructed.
    cpt1 = linear_comb(1.0, cpt0, +ribbon_width[ss], d)
    spline_side1.set_c_point(3, cpt1)

    cpt2 = linear_comb(1.0, cpt0, -ribbon_width[ss], d)
    spline_side2.set_c_point(3, cpt2)

    # Saving the plane vector (for the flipping test in the next call).
    flip_test_v.set(d)


def construct_control_points(residues, res, ss, handness):
    p1 = Vec3()
    p2 = Vec3()
    p3 = Vec3()

    if res == 0:
        # The control points 2 and 3 are created.
        flip_test_v.set(0, 0, 0)

        res0 = residues[res]
        res1 = residues[res + 1]
        ca0 = res0["CA"]
        ox0 = res0["O"]
        ca1 = res1["CA"]
        add_control_points(ca0, ox0, ca1, ss, handness)
        spline_side1.copy_c_points(3, 2)
        spline_center.copy_c_points(3, 2)
        spline_side2.copy_c_points(3, 2)

        res0 = residues[res + 1]
        res1 = residues[res + 2]
        ca0 = res0["CA"]
        ox0 = res0["O"]
        ca1 = res1["CA"]
        add_control_points(ca0, ox0, ca1, ss, handness)

        # We still need the two first control points.
        # Moving backwards along the cp_center[2] - cp_center[3] direction.
        spline_center.get_c_point(2, p2)
        spline_center.get_c_point(3, p3)

        p1 = linear_comb(2.0, p2, -1, p3)
        spline_center.set_c_point(1, p1)
        spline_side1.set_c_point(1, linear_comb(1.0, p1, +ribbon_width[ss], flip_test_v))
        spline_side2.set_c_point(1, linear_comb(1.0, p1, -ribbon_width[ss], flip_test_v))

        p0 = linear_comb(2.0, p1, -1, p2)
        spline_center.set_c_point(0, p0)
        spline_side1.set_c_point(0, linear_comb(1.0, p0, +ribbon_width[ss], flip_test_v))
        spline_side2.set_c_point(0, linear_comb(1.0, p0, -ribbon_width[ss], flip_test_v))
    else:
        shift_control_points()
        if len(residues) - 1 == res or len(residues) - 2 == res:
            # Moving forward along the cp_center[1] - cp_center[2] direction.
            spline_center.get_c_point(1, p1)
            spline_center.get_c_point(2, p2)

            p3 = linear_comb(2.0, p2, -1, p1)
            spline_center.set_c_point(3, p3)
            spline_side1.set_c_point(3, linear_comb(1.0, p3, +ribbon_width[ss], flip_test_v))
            spline_side2.set_c_point(3, linear_comb(1.0, p3, -ribbon_width[ss], flip_test_v))
        else:
            res0 = residues[res + 1]
            res1 = residues[res + 2]
            ca0 = res0["CA"]
            ox0 = res0["O"]
            ca1 = res1["CA"]
            add_control_points(ca0, ox0, ca1, ss, handness)
    spline_side1.update_matrix3()
    spline_center.update_matrix3()
    spline_side2.update_matrix3()


def linear_comb(scalar0, vector0, scalar1, vector1):
    return vector0 * scalar0 + vector1 * scalar1


# ---------------------------------------------------------------- PDB


def read_pdb(filename):
    global models, model_shift

    # Data files are read relative to the sketch's folder
    with open(os.path.join(os.path.dirname(__file__), filename)) as f:
        str_lines = f.read().splitlines()

    models = []

    xmin = ymin = zmin = 10000
    xmax = ymax = zmax = -10000

    atoms = None
    residues = None
    residue = None
    res0 = -1
    nmdl = -1
    for s in str_lines:
        if s.startswith("MODEL") or (s.startswith("ATOM") and res0 == -1):
            nmdl += 1

            res0 = -1

            atoms = []
            residues = []

        if s.startswith("ATOM"):
            atstr = s[12:15].strip()
            resstr = s[22:26].strip()
            res = int(resstr)

            xstr = s[30:37].strip()
            ystr = s[38:45].strip()
            zstr = s[46:53].strip()

            x = scale_factor * float(xstr)
            y = scale_factor * float(ystr)
            z = scale_factor * float(zstr)
            v = Vec3(x, y, z)

            xmin = min(xmin, x)
            xmax = max(xmax, x)

            ymin = min(ymin, y)
            ymax = max(ymax, y)

            zmin = min(zmin, z)
            zmax = max(zmax, z)

            atoms.append(v)

            if res0 != res:
                if residue is not None:
                    residues.append(residue)
                residue = {}
            residue[atstr] = v

            res0 = res

        if s.startswith("ENDMDL") or s.startswith("TER"):
            if residue is not None:
                residues.append(residue)

            create_ribbon_model(residues, models)
            rgyr = calculate_gyr_radius(atoms)

            res0 = -1
            residue = None
            atoms = None
            residues = None

    if residue is not None:
        residues.append(residue)

        create_ribbon_model(residues, models)
        rgyr = calculate_gyr_radius(atoms)

        atoms = None
        residues = None

    # Centering models at (0, 0, 0).
    dx = -0.5 * (xmin + xmax)
    dy = -0.5 * (ymin + ymax)
    dz = -0.5 * (zmin + zmax)
    model_shift = Vec3(dx, dy, dz)

    print("Loaded PDB file with " + str(len(models)) + " models.")


run()
