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
    "pixel_width",
    "pixel_height",
    "pixel_density",
)
_DYNAMIC_FUNCTIONS = (
    "mouse_x",
    "mouse_y",
    "pmouse_x",
    "pmouse_y",
    "frame_count",
    "delta_time",
    "elapsed_time",
)
_DYNAMIC = (
    _DYNAMIC_GRAPHICS_ATTRS + _DYNAMIC_FUNCTIONS + (
        "mouse_is_pressed",
        "mouse_button",
        "moved_x",
        "moved_y",
        "mouse_wheel",
        "key",
        "key_code",
        "key_is_pressed",
        "display_width",
        "display_height",
    )
)


def _get_graphics():
    return getattr(_native, "_graphics", None)


def __getattr__(name):
    if name in _DYNAMIC_GRAPHICS_ATTRS:
        g = _get_graphics()
        if g is not None:
            return getattr(g, name)
    if name in _DYNAMIC_FUNCTIONS:
        fn = getattr(_native, name, None)
        if callable(fn):
            return fn()
    if name in ("display_width", "display_height"):
        mon = getattr(_native, "primary_monitor", lambda: None)()
        if mon is None:
            return 0
        return mon.width if name == "display_width" else mon.height
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def __dir__():
    return sorted(set(list(globals().keys()) + list(_DYNAMIC)))


del _sys, _name, _sub
