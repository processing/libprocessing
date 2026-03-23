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
del _sys, _native, _name, _sub
