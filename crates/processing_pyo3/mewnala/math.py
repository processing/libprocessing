"""Processing math methods and vector/quaternion types."""
from .mewnala import math as _native_math
from math import (
    sin, cos, tan,
    asin, acos, atan, atan2,
    sqrt, exp, log,
    ceil, floor,
    degrees, radians,
)

Vec2 = _native_math.Vec2
Vec3 = _native_math.Vec3
Vec4 = _native_math.Vec4
Quat = _native_math.Quat
VecIter = _native_math.PyVecIter
vec2 = _native_math.vec2
vec3 = _native_math.vec3
vec4 = _native_math.vec4
quat = _native_math.quat


def sq(x):
    return x * x


def pow(base, exponent):
    return base ** exponent


def constrain(value, low, high):
    if value < low:
        return low
    if value > high:
        return high
    return value


def lerp(start, stop, amt):
    return start + (stop - start) * amt


def norm(value, start, stop):
    return (value - start) / (stop - start)


def remap(value, start1, stop1, start2, stop2):
    return start2 + (stop2 - start2) * ((value - start1) / (stop1 - start1))


def mag(*args):
    if len(args) == 2:
        a, b = args
        return sqrt(a * a + b * b)
    if len(args) == 3:
        a, b, c = args
        return sqrt(a * a + b * b + c * c)
    raise TypeError(f"mag() takes 2 or 3 arguments ({len(args)} given)")


def dist(*args):
    if len(args) == 4:
        x1, y1, x2, y2 = args
        dx, dy = x2 - x1, y2 - y1
        return sqrt(dx * dx + dy * dy)
    if len(args) == 6:
        x1, y1, z1, x2, y2, z2 = args
        dx, dy, dz = x2 - x1, y2 - y1, z2 - z1
        return sqrt(dx * dx + dy * dy + dz * dz)
    raise TypeError(f"dist() takes 4 or 6 arguments ({len(args)} given)")
