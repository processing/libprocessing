"""Processing math methods and vector/quaternion types."""
import math as _math
import random as _random
from .mewnala import math as _native_math

# extend native math so glob imports work
for _name in dir(_native_math):
    if not _name.startswith("_"):
        globals()[_name] = getattr(_native_math, _name)
del _name
from math import (
    sin, cos, tan,
    atan, atan2,
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

_NAN = float("nan")
_INF = float("inf")


def _safe_div(num, den):
    if den == 0:
        if num == 0:
            return _NAN
        return _INF if num > 0 else -_INF
    return num / den


def sq(x):
    return x * x


def pow(base, exponent):
    if base < 0 and not float(exponent).is_integer():
        return _NAN
    try:
        return base ** exponent
    except (OverflowError, ZeroDivisionError):
        return _INF


def sqrt(x):
    if x < 0:
        return _NAN
    return _math.sqrt(x)


def exp(x):
    try:
        return _math.exp(x)
    except OverflowError:
        return _INF


def log(x):
    if x == 0:
        return -_INF
    if x < 0:
        return _NAN
    return _math.log(x)


def asin(x):
    if x < -1 or x > 1:
        return _NAN
    return _math.asin(x)


def acos(x):
    if x < -1 or x > 1:
        return _NAN
    return _math.acos(x)


def round(x):
    return _math.floor(x + 0.5)


def constrain(value, low, high):
    if value < low:
        return low
    if value > high:
        return high
    return value


def lerp(start, stop, amt):
    return start + (stop - start) * amt


def norm(value, start, stop):
    return _safe_div(value - start, stop - start)


def remap(value, start1, stop1, start2, stop2, within_bounds=False):
    mapped = start2 + (stop2 - start2) * _safe_div(value - start1, stop1 - start1)
    if not within_bounds:
        return mapped
    if start2 < stop2:
        return constrain(mapped, start2, stop2)
    return constrain(mapped, stop2, start2)


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


def random(low, high=None):
    """`random(high)` is in [0, high), `random(low, high)` in [low, high)."""
    if high is None:
        low, high = 0.0, low
    return low + _random.random() * (high - low)


def random_gaussian():
    return _random.gauss(0.0, 1.0)


def random_seed(seed):
    _random.seed(seed)


def bezier_point(a, b, c, d, t):
    u = 1.0 - t
    return a * u * u * u + 3 * b * t * u * u + 3 * c * t * t * u + d * t * t * t


def bezier_tangent(a, b, c, d, t):
    return 3 * t * t * (-a + 3 * b - 3 * c + d) + 6 * t * (a - 2 * b + c) + 3 * (-a + b)


# Catmull-Rom, matching `curve()` (Processing's curveTightness(0))
def curve_point(a, b, c, d, t):
    return 0.5 * (
        2 * b
        + (-a + c) * t
        + (2 * a - 5 * b + 4 * c - d) * t * t
        + (-a + 3 * b - 3 * c + d) * t * t * t
    )


def curve_tangent(a, b, c, d, t):
    return 0.5 * ((-a + c) + 2 * (2 * a - 5 * b + 4 * c - d) * t + 3 * (-a + 3 * b - 3 * c + d) * t * t)
