# M_3_4_02
#
# Generative Gestaltung, ISBN: 978-3-87439-759-9
# First Edition, Hermann Schmidt, Mainz, 2009
# Hartmut Bohnacker, Benedikt Gross, Julia Laub, Claudius Lazzeroni
# Copyright 2009 Hartmut Bohnacker, Benedikt Gross, Julia Laub, Claudius Lazzeroni
#
# http://www.generative-gestaltung.de
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# Ported from the Processing original to processing_py.

# draws multiple meshes, all parts of a sphere
#
# MOUSE
# click + drag        : rotate
from processing_py import *


class Mesh:

    # ------ constants ------

    CUSTOM = 0
    PLANE = CUSTOM
    TUBE = 1
    SPHERE = 2
    TORUS = 3
    PARABOLOID = 4
    STEINBACHSCREW = 5
    SINE = 6
    FIGURE8TORUS = 7
    ELLIPTICTORUS = 8
    CORKSCREW = 9
    BOHEMIANDOME = 10
    BOW = 11
    MAEDERSOWL = 12
    ASTROIDALELLIPSOID = 13
    TRIAXIALTRITORUS = 14
    LIMPETTORUS = 15
    HORN = 16
    SHELL = 17
    KIDNEY = 18
    LEMNISCAPE = 19
    TRIANGULOID = 20
    SUPERFORMULA = 21

    # ------ construktors ------

    # Mesh()
    # Mesh(the_form)
    # Mesh(the_form, the_u_num, the_v_num)
    # Mesh(the_form, the_u_min, the_u_max, the_v_min, the_v_max)
    # Mesh(the_form, the_u_num, the_v_num, the_u_min, the_u_max, the_v_min, the_v_max)
    def __init__(self, *args):

        # ------ mesh parameters ------

        self.form = Mesh.PARABOLOID

        self.u_min = -PI
        self.u_max = PI
        self.u_count = 50

        self.v_min = -PI
        self.v_max = PI
        self.v_count = 50

        self.params = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]

        self.draw_mode = TRIANGLE_STRIP
        self.min_hue = 0
        self.max_hue = 0
        self.min_saturation = 0
        self.max_saturation = 0
        self.min_brightness = 50
        self.max_brightness = 50
        self.mesh_alpha = 100

        self.mesh_distortion = 0

        self.points = []

        if len(args) == 0:
            self.form = Mesh.CUSTOM
        else:
            the_form = args[0]
            if the_form >= 0:
                self.form = the_form
        if len(args) == 3:
            self.u_count = max(args[1], 1)
            self.v_count = max(args[2], 1)
        if len(args) == 5:
            self.u_min = args[1]
            self.u_max = args[2]
            self.v_min = args[3]
            self.v_max = args[4]
        if len(args) == 7:
            self.u_count = max(args[1], 1)
            self.v_count = max(args[2], 1)
            self.u_min = args[3]
            self.u_max = args[4]
            self.v_min = args[5]
            self.v_max = args[6]
        self.update()

    # ------ calculate points ------

    def update(self):
        self.points = []

        for iv in range(self.v_count + 1):
            row = []
            for iu in range(self.u_count + 1):
                u = remap(iu, 0, self.u_count, self.u_min, self.u_max)
                v = remap(iv, 0, self.v_count, self.v_min, self.v_max)

                if self.form == Mesh.CUSTOM:
                    row.append(self.calculate_points(u, v))
                elif self.form == Mesh.TUBE:
                    row.append(self.tube(u, v))
                elif self.form == Mesh.SPHERE:
                    row.append(self.sphere(u, v))
                elif self.form == Mesh.TORUS:
                    row.append(self.torus(u, v))
                elif self.form == Mesh.PARABOLOID:
                    row.append(self.paraboloid(u, v))
                elif self.form == Mesh.STEINBACHSCREW:
                    row.append(self.steinbach_screw(u, v))
                elif self.form == Mesh.SINE:
                    row.append(self.sine(u, v))
                elif self.form == Mesh.FIGURE8TORUS:
                    row.append(self.figure8_torus(u, v))
                elif self.form == Mesh.ELLIPTICTORUS:
                    row.append(self.elliptic_torus(u, v))
                elif self.form == Mesh.CORKSCREW:
                    row.append(self.corkscrew(u, v))
                elif self.form == Mesh.BOHEMIANDOME:
                    row.append(self.bohemian_dome(u, v))
                elif self.form == Mesh.BOW:
                    row.append(self.bow(u, v))
                elif self.form == Mesh.MAEDERSOWL:
                    row.append(self.maeders_owl(u, v))
                elif self.form == Mesh.ASTROIDALELLIPSOID:
                    row.append(self.astroidal_ellipsoid(u, v))
                elif self.form == Mesh.TRIAXIALTRITORUS:
                    row.append(self.triaxial_tritorus(u, v))
                elif self.form == Mesh.LIMPETTORUS:
                    row.append(self.limpet_torus(u, v))
                elif self.form == Mesh.HORN:
                    row.append(self.horn(u, v))
                elif self.form == Mesh.SHELL:
                    row.append(self.shell(u, v))
                elif self.form == Mesh.KIDNEY:
                    row.append(self.kidney(u, v))
                elif self.form == Mesh.LEMNISCAPE:
                    row.append(self.lemniscape(u, v))
                elif self.form == Mesh.TRIANGULOID:
                    row.append(self.trianguloid(u, v))
                elif self.form == Mesh.SUPERFORMULA:
                    row.append(self.superformula(u, v))
                else:
                    row.append(self.calculate_points(u, v))
            self.points.append(row)

    # ------ getters and setters ------

    def get_form(self):
        return self.form

    def set_form(self, the_value):
        self.form = the_value

    def get_form_name(self):
        if self.form == Mesh.CUSTOM:
            return "Custom"
        elif self.form == Mesh.TUBE:
            return "Tube"
        elif self.form == Mesh.SPHERE:
            return "Sphere"
        elif self.form == Mesh.TORUS:
            return "Torus"
        elif self.form == Mesh.PARABOLOID:
            return "Paraboloid"
        elif self.form == Mesh.STEINBACHSCREW:
            return "Steinbach Screw"
        elif self.form == Mesh.SINE:
            return "Sine"
        elif self.form == Mesh.FIGURE8TORUS:
            return "Figure 8 Torus"
        elif self.form == Mesh.ELLIPTICTORUS:
            return "Elliptic Torus"
        elif self.form == Mesh.CORKSCREW:
            return "Corkscrew"
        elif self.form == Mesh.BOHEMIANDOME:
            return "Bohemian Dome"
        elif self.form == Mesh.BOW:
            return "Bow"
        elif self.form == Mesh.MAEDERSOWL:
            return "Maeders Owl"
        elif self.form == Mesh.ASTROIDALELLIPSOID:
            return "Astoidal Ellipsoid"
        elif self.form == Mesh.TRIAXIALTRITORUS:
            return "Triaxial Tritorus"
        elif self.form == Mesh.LIMPETTORUS:
            return "Limpet Torus"
        elif self.form == Mesh.HORN:
            return "Horn"
        elif self.form == Mesh.SHELL:
            return "Shell"
        elif self.form == Mesh.KIDNEY:
            return "Kidney"
        elif self.form == Mesh.LEMNISCAPE:
            return "Lemniscape"
        elif self.form == Mesh.TRIANGULOID:
            return "Trianguloid"
        elif self.form == Mesh.SUPERFORMULA:
            return "Superformula"
        return ""

    def get_u_min(self):
        return self.u_min

    def set_u_min(self, the_value):
        self.u_min = the_value

    def get_u_max(self):
        return self.u_max

    def set_u_max(self, the_value):
        self.u_max = the_value

    def get_u_count(self):
        return self.u_count

    def set_u_count(self, the_value):
        self.u_count = the_value

    def get_v_min(self):
        return self.v_min

    def set_v_min(self, the_value):
        self.v_min = the_value

    def get_v_max(self):
        return self.v_max

    def set_v_max(self, the_value):
        self.v_max = the_value

    def get_v_count(self):
        return self.v_count

    def set_v_count(self, the_value):
        self.v_count = the_value

    def get_params(self):
        return self.params

    def set_params(self, the_values):
        self.params = the_values

    def get_param(self, the_index):
        return self.params[the_index]

    def set_param(self, the_index, the_value):
        self.params[the_index] = the_value

    def get_draw_mode(self):
        return self.draw_mode

    def set_draw_mode(self, the_mode):
        self.draw_mode = the_mode

    def get_mesh_distortion(self):
        return self.mesh_distortion

    def set_mesh_distortion(self, the_value):
        self.mesh_distortion = the_value

    def set_color_range(self, the_min_hue, the_max_hue, the_min_saturation, the_max_saturation, the_min_brightness, the_max_brightness, the_mesh_alpha):
        self.min_hue = the_min_hue
        self.max_hue = the_max_hue
        self.min_saturation = the_min_saturation
        self.max_saturation = the_max_saturation
        self.min_brightness = the_min_brightness
        self.max_brightness = the_max_brightness
        self.mesh_alpha = the_mesh_alpha

    def get_min_hue(self):
        return self.min_hue

    def set_min_hue(self, min_hue):
        self.min_hue = min_hue

    def get_max_hue(self):
        return self.max_hue

    def set_max_hue(self, max_hue):
        self.max_hue = max_hue

    def get_min_saturation(self):
        return self.min_saturation

    def set_min_saturation(self, min_saturation):
        self.min_saturation = min_saturation

    def get_max_saturation(self):
        return self.max_saturation

    def set_max_saturation(self, max_saturation):
        self.max_saturation = max_saturation

    def get_min_brightness(self):
        return self.min_brightness

    def set_min_brightness(self, min_brightness):
        self.min_brightness = min_brightness

    def get_max_brightness(self):
        return self.max_brightness

    def set_max_brightness(self, max_brightness):
        self.max_brightness = max_brightness

    def get_mesh_alpha(self):
        return self.mesh_alpha

    def set_mesh_alpha(self, mesh_alpha):
        self.mesh_alpha = mesh_alpha

    # ------ functions for calculating the mesh points ------

    def calculate_points(self, u, v):
        x = u
        y = v
        z = 0

        return Vec3(x, y, z)

    def default_form(self, u, v):
        x = u
        y = v
        z = 0

        return Vec3(x, y, z)

    def tube(self, u, v):
        x = sin(u)
        y = self.params[0] * v
        z = cos(u)

        return Vec3(x, y, z)

    def sphere(self, u, v):
        v /= 2
        v += HALF_PI
        x = 2 * (sin(v) * sin(u))
        y = 2 * (self.params[0] * cos(v))
        z = 2 * (sin(v) * cos(u))

        return Vec3(x, y, z)

    def torus(self, u, v):
        x = 1 * ((self.params[1] + 1 + self.params[0] * cos(v)) * sin(u))
        y = 1 * (self.params[0] * sin(v))
        z = 1 * ((self.params[1] + 1 + self.params[0] * cos(v)) * cos(u))

        return Vec3(x, y, z)

    def paraboloid(self, u, v):
        pd = self.params[0]
        if pd == 0:
            pd = 0.0001
        x = self.power((v / pd), 0.5) * sin(u)
        y = v
        z = self.power((v / pd), 0.5) * cos(u)

        return Vec3(x, y, z)

    def steinbach_screw(self, u, v):
        x = u * cos(v)
        y = u * sin(self.params[0] * v)
        z = v * cos(u)

        return Vec3(x, y, z)

    def sine(self, u, v):
        x = 2 * sin(u)
        y = 2 * sin(self.params[0] * v)
        z = 2 * sin(u + v)

        return Vec3(x, y, z)

    def figure8_torus(self, u, v):
        x = 1.5 * cos(u) * (self.params[0] + sin(v) * cos(u) - sin(2 * v) * sin(u) / 2)
        y = 1.5 * sin(u) * (self.params[0] + sin(v) * cos(u) - sin(2 * v) * sin(u) / 2)
        z = 1.5 * sin(u) * sin(v) + cos(u) * sin(2 * v) / 2

        return Vec3(x, y, z)

    def elliptic_torus(self, u, v):
        x = 1.5 * (self.params[0] + cos(v)) * cos(u)
        y = 1.5 * (self.params[0] + cos(v)) * sin(u)
        z = 1.5 * sin(v) + cos(v)

        return Vec3(x, y, z)

    def corkscrew(self, u, v):
        x = cos(u) * cos(v)
        y = sin(u) * cos(v)
        z = sin(v) + self.params[0] * u

        return Vec3(x, y, z)

    def bohemian_dome(self, u, v):
        x = 2 * cos(u)
        y = 2 * sin(u) + self.params[0] * cos(v)
        z = 2 * sin(v)

        return Vec3(x, y, z)

    def bow(self, u, v):
        u /= TWO_PI
        v /= TWO_PI
        x = (2 + self.params[0] * sin(TWO_PI * u)) * sin(2 * TWO_PI * v)
        y = (2 + self.params[0] * sin(TWO_PI * u)) * cos(2 * TWO_PI * v)
        z = self.params[0] * cos(TWO_PI * u) + 3 * cos(TWO_PI * v)

        return Vec3(x, y, z)

    def maeders_owl(self, u, v):
        x = 0.4 * (v * cos(u) - 0.5 * self.params[0] * self.power(v, 2) * cos(2 * u))
        y = 0.4 * (-v * sin(u) - 0.5 * self.params[0] * self.power(v, 2) * sin(2 * u))
        z = 0.4 * (4 * self.power(v, 1.5) * cos(3 * u / 2) / 3)

        return Vec3(x, y, z)

    def astroidal_ellipsoid(self, u, v):
        u /= 2
        x = 3 * self.power(cos(u) * cos(v), 3 * self.params[0])
        y = 3 * self.power(sin(u) * cos(v), 3 * self.params[0])
        z = 3 * self.power(sin(v), 3 * self.params[0])

        return Vec3(x, y, z)

    def triaxial_tritorus(self, u, v):
        x = 1.5 * sin(u) * (1 + cos(v))
        y = 1.5 * sin(u + TWO_PI / 3 * self.params[0]) * (1 + cos(v + TWO_PI / 3 * self.params[0]))
        z = 1.5 * sin(u + 2 * TWO_PI / 3 * self.params[0]) * (1 + cos(v + 2 * TWO_PI / 3 * self.params[0]))

        return Vec3(x, y, z)

    def limpet_torus(self, u, v):
        x = 1.5 * self.params[0] * cos(u) / (sqrt(2) + sin(v))
        y = 1.5 * self.params[0] * sin(u) / (sqrt(2) + sin(v))
        z = 1.5 * 1 / (sqrt(2) + cos(v))

        return Vec3(x, y, z)

    def horn(self, u, v):
        u /= PI
        # v /= PI
        x = (2 * self.params[0] + u * cos(v)) * sin(TWO_PI * u)
        y = (2 * self.params[0] + u * cos(v)) * cos(TWO_PI * u) + 2 * u
        z = u * sin(v)

        return Vec3(x, y, z)

    def shell(self, u, v):
        x = self.params[1] * (1 - (u / TWO_PI)) * cos(self.params[0] * u) * (1 + cos(v)) + self.params[3] * cos(self.params[0] * u)
        y = self.params[1] * (1 - (u / TWO_PI)) * sin(self.params[0] * u) * (1 + cos(v)) + self.params[3] * sin(self.params[0] * u)
        z = self.params[2] * (u / TWO_PI) + self.params[0] * (1 - (u / TWO_PI)) * sin(v)

        return Vec3(x, y, z)

    def kidney(self, u, v):
        u /= 2
        x = cos(u) * (self.params[0] * 3 * cos(v) - cos(3 * v))
        y = sin(u) * (self.params[0] * 3 * cos(v) - cos(3 * v))
        z = 3 * sin(v) - sin(3 * v)

        return Vec3(x, y, z)

    def lemniscape(self, u, v):
        u /= 2
        cosv_sqrt_abs_sin2u = cos(v) * sqrt(abs(sin(2 * self.params[0] * u)))
        x = cosv_sqrt_abs_sin2u * cos(u)
        y = cosv_sqrt_abs_sin2u * sin(u)
        z = 3 * (self.power(x, 2) - self.power(y, 2) + 2 * x * y * self.power(tan(v), 2))
        x *= 3
        y *= 3
        return Vec3(x, y, z)

    def trianguloid(self, u, v):
        x = 0.75 * (sin(3 * u) * 2 / (2 + cos(v)))
        y = 0.75 * ((sin(u) + 2 * self.params[0] * sin(2 * u)) * 2 / (2 + cos(v + TWO_PI)))
        z = 0.75 * ((cos(u) - 2 * self.params[0] * cos(2 * u)) * (2 + cos(v)) * ((2 + cos(v + TWO_PI / 3)) * 0.25))

        return Vec3(x, y, z)

    def superformula(self, u, v):
        v /= 2

        # Superformel 1
        a = self.params[0]
        b = self.params[1]
        m = self.params[2]
        n1 = self.params[3]
        n2 = self.params[4]
        n3 = self.params[5]
        r1 = pow(pow(abs(cos(m * u / 4) / a), n2) + pow(abs(sin(m * u / 4) / b), n3), -1 / n1)

        # Superformel 2
        a = self.params[6]
        b = self.params[7]
        m = self.params[8]
        n1 = self.params[9]
        n2 = self.params[10]
        n3 = self.params[11]
        r2 = pow(pow(abs(cos(m * v / 4) / a), n2) + pow(abs(sin(m * v / 4) / b), n3), -1 / n1)

        x = 2 * (r1 * sin(u) * r2 * cos(v))
        y = 2 * (r2 * sin(v))
        z = 2 * (r1 * cos(u) * r2 * cos(v))

        return Vec3(x, y, z)

    # ------ definition of some mathematical functions ------

    # the processing-function pow works a bit differently for negative bases
    def power(self, b, e):
        if b >= 0 or int(e) == e:
            return pow(b, e)
        else:
            return -pow(-b, e)

    def log_e(self, v):
        if v >= 0:
            return log(v)
        else:
            return -log(-v)

    def sinh(self, a):
        return sin(HALF_PI / 2 - a)

    def cosh(self, a):
        return cos(HALF_PI / 2 - a)

    def tanh(self, a):
        return tan(HALF_PI / 2 - a)

    # ------ draw mesh ------

    def draw(self):
        points = self.points

        if self.draw_mode == QUADS or self.draw_mode == TRIANGLES:
            iu_max = self.u_count - 1
            iv_max = self.v_count - 1
        else:
            iu_max = self.u_count
            iv_max = self.v_count - 1

        # store previously set style
        push_style()

        min_h = self.min_hue
        max_h = self.max_hue
        if abs(max_h - min_h) < 20:
            max_h = min_h
        min_s = self.min_saturation
        max_s = self.max_saturation
        if abs(max_s - min_s) < 10:
            max_s = min_s
        min_b = self.min_brightness
        max_b = self.max_brightness
        if abs(max_b - min_b) < 10:
            max_b = min_b

        for iv in range(iv_max + 1):
            if self.draw_mode == TRIANGLES:

                for iu in range(iu_max + 1):
                    fill(hsva(random(min_h, max_h), random(min_s, max_s) / 100, random(min_b, max_b) / 100, self.mesh_alpha / 100))
                    begin_shape(self.draw_mode)
                    r1 = self.mesh_distortion * random(-1, 1)
                    r2 = self.mesh_distortion * random(-1, 1)
                    r3 = self.mesh_distortion * random(-1, 1)
                    vertex(points[iv][iu].x + r1, points[iv][iu].y + r2, points[iv][iu].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    vertex(points[iv + 1][iu + 1].x + r1, points[iv + 1][iu + 1].y + r2, points[iv + 1][iu + 1].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    vertex(points[iv + 1][iu].x + r1, points[iv + 1][iu].y + r2, points[iv + 1][iu].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    end_shape()

                    fill(hsva(random(min_h, max_h), random(min_s, max_s) / 100, random(min_b, max_b) / 100, self.mesh_alpha / 100))
                    begin_shape(self.draw_mode)
                    r1 = self.mesh_distortion * random(-1, 1)
                    r2 = self.mesh_distortion * random(-1, 1)
                    r3 = self.mesh_distortion * random(-1, 1)
                    vertex(points[iv + 1][iu + 1].x + r1, points[iv + 1][iu + 1].y + r2, points[iv + 1][iu + 1].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    vertex(points[iv][iu].x + r1, points[iv][iu].y + r2, points[iv][iu].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    vertex(points[iv][iu + 1].x + r1, points[iv][iu + 1].y + r2, points[iv][iu + 1].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    end_shape()

            elif self.draw_mode == QUADS:
                for iu in range(iu_max + 1):
                    fill(hsva(random(min_h, max_h), random(min_s, max_s) / 100, random(min_b, max_b) / 100, self.mesh_alpha / 100))
                    begin_shape(self.draw_mode)

                    r1 = self.mesh_distortion * random(-1, 1)
                    r2 = self.mesh_distortion * random(-1, 1)
                    r3 = self.mesh_distortion * random(-1, 1)
                    vertex(points[iv][iu].x + r1, points[iv][iu].y + r2, points[iv][iu].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    vertex(points[iv + 1][iu].x + r1, points[iv + 1][iu].y + r2, points[iv + 1][iu].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    vertex(points[iv + 1][iu + 1].x + r1, points[iv + 1][iu + 1].y + r2, points[iv + 1][iu + 1].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    vertex(points[iv][iu + 1].x + r1, points[iv][iu + 1].y + r2, points[iv][iu + 1].z + r3)  # GAP: vertex(x, y, z) in immediate mode

                    end_shape()
            else:
                # Draw Strips
                fill(hsva(random(min_h, max_h), random(min_s, max_s) / 100, random(min_b, max_b) / 100, self.mesh_alpha / 100))
                begin_shape(self.draw_mode)

                for iu in range(iu_max + 1):
                    r1 = self.mesh_distortion * random(-1, 1)
                    r2 = self.mesh_distortion * random(-1, 1)
                    r3 = self.mesh_distortion * random(-1, 1)
                    vertex(points[iv][iu].x + r1, points[iv][iu].y + r2, points[iv][iu].z + r3)  # GAP: vertex(x, y, z) in immediate mode
                    vertex(points[iv + 1][iu].x + r1, points[iv + 1][iu].y + r2, points[iv + 1][iu].z + r3)  # GAP: vertex(x, y, z) in immediate mode

                end_shape()

        pop_style()


# parameters
mesh_count = 6
my_meshes = []

form = Mesh.SPHERE

u_count = 8
v_count = 4

# rotation
offset_x = 0
offset_y = 0
click_x = 0
click_y = 0
rotation_x = 0
rotation_y = 0
target_rotation_x = 0
target_rotation_y = 0
click_rotation_x = 0
click_rotation_y = 0


def setup():
    size(800, 800)
    mode_3d()

    # ------ initialize meshes ------
    random_seed(35976)

    for i in range(mesh_count):
        u_min = random(-6, 6)
        u_max = u_min + random(2, 3)

        v_min = random(-6, 6)
        v_max = v_min + random(1, 2)

        my_meshes.append(Mesh(form, u_count, v_count, u_min, u_max, v_min, v_max))
        my_meshes[i].set_draw_mode(QUAD_STRIP)

        col = random(160, 240)
        my_meshes[i].set_color_range(col, col, 0, 0, 100, 100, 100)


def draw():
    # ------ setup canvas, lights and view ------
    # setup drawing style
    background(1.0)
    stroke_weight(1 / 130.0)

    set_view()

    scale(130)

    # ------ draw meshes ------
    random_seed(0)
    for i in range(mesh_count):
        push_matrix()
        scale(random(0.9, 1.2))
        my_meshes[i].draw()
        pop_matrix()


def mouse_pressed():
    global click_x, click_y, click_rotation_x, click_rotation_y
    click_x = mouse_x
    click_y = mouse_y
    click_rotation_x = rotation_x
    click_rotation_y = rotation_y


def set_view():
    global offset_x, offset_y, target_rotation_x, target_rotation_y, rotation_x, rotation_y
    # translate(width * 0.5, height * 0.5) is not needed: the 3D origin is the canvas center

    if mouse_is_pressed:
        offset_x = mouse_x - click_x
        offset_y = mouse_y - click_y
        target_rotation_x = click_rotation_x + offset_x / width * TWO_PI
        target_rotation_y = min(max(click_rotation_y + offset_y / height * TWO_PI, -HALF_PI), HALF_PI)
        rotation_x += (target_rotation_x - rotation_x) * 0.25
        rotation_y += (target_rotation_y - rotation_y) * 0.25
    rotate_x(-rotation_y)
    rotate_y(rotation_x)


run()
