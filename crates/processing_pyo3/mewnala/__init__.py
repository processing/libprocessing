from .mewnala import *

# re-export the native submodules as submodules of this module, if they exist
# this allows users to import from `mewnala.math` and `mewnala.color`
# if they exist, without needing to know about the internal structure of the native module
import sys as _sys
from . import mewnala as _native
for _name in ("math", "color"):
    _sub = getattr(_native, _name, None)
    if _sub is not None:
        _sys.modules[f"{__name__}.{_name}"] = _sub

# global var handling. for wildcard import of our module, we copy into globals, otherwise
# we dispatch to get attr and call the underlying getter method 

_DYNAMIC_GRAPHICS_ATTRS = (
    "width",
    "height",
    "focused",
    "pixel_density",
    "pixel_width",
    "pixel_height",
    "mouse_x",
    "mouse_y",
    "pmouse_x",
    "pmouse_y",
    "mouse_is_pressed",
    "mouse_button",
    "mouse_wheel",
    "moved_x",
    "moved_y",
    "key",
    "key_code",
    "key_is_pressed",
)

_DYNAMIC_TIME_ATTRS = (
    "frame_count",
    "delta_time",
    "elapsed_time",
)

_DEFAULT_GRAPHICS_VALUES = {
    "width": 100,
    "height": 100,
    "focused": False,
    "pixel_density": 1.0,
    "pixel_width": 100,
    "pixel_height": 100,
    "mouse_x": 0.0,
    "mouse_y": 0.0,
    "pmouse_x": 0.0,
    "pmouse_y": 0.0,
    "mouse_is_pressed": False,
    "mouse_button": None,
    "mouse_wheel": 0.0,
    "moved_x": 0.0,
    "moved_y": 0.0,
    "key": None,
    "key_code": None,
    "key_is_pressed": False,
}

_DYNAMIC = (
    _DYNAMIC_GRAPHICS_ATTRS
    + _DYNAMIC_TIME_ATTRS
    + ("display_width", "display_height", "window_x", "window_y")
)


def _get_graphics():
    return getattr(_native, "_graphics", None)


def __getattr__(name):
    if name in _DYNAMIC_GRAPHICS_ATTRS:
        g = _get_graphics()
        if g is not None:
            return getattr(g, name)
        return _DEFAULT_GRAPHICS_VALUES[name]
    if name in _DYNAMIC_TIME_ATTRS:
        fn = getattr(_native, f"_dyn_{name}", None)
        if not callable(fn):
            return 0
        try:
            return fn()
        except RuntimeError:
            return 0 if name == "frame_count" else 0.0
    if name in ("display_width", "display_height"):
        try:
            mon = getattr(_native, "primary_monitor", lambda: None)()
        except RuntimeError:
            return 0
        if mon is None:
            return 0
        return mon.width if name == "display_width" else mon.height
    if name in ("window_x", "window_y"):
        g = _get_graphics()
        if g is None:
            return 0
        x, y = g.surface.position
        return x if name == "window_x" else y
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def __dir__():
    return sorted(set(list(globals().keys()) + list(_DYNAMIC)))


__all__ = sorted(
    {n for n in dir(_native) if not n.startswith("_")} | set(_DYNAMIC)
)

del _sys, _name, _sub
